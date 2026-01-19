use std::ops::Range;

use noise_functions::{OpenSimplex2, Sample};

const SCALE: [i8; 7] = [60, 62, 64, 65, 67, 69, 71]; // C major in semitones
const MARKOV_SCALE: [[f32; 7]; 7] = [
    [0.3781, 0.1806, 0.0823, 0.0398, 0.0872, 0.0893, 0.1428],
    [0.2310, 0.3489, 0.1973, 0.0418, 0.0718, 0.0497, 0.0596],
    [0.0998, 0.2476, 0.3369, 0.1331, 0.1063, 0.0543, 0.0219],
    [0.0545, 0.0824, 0.2558, 0.3531, 0.1737, 0.0693, 0.0113],
    [0.0943, 0.0639, 0.1080, 0.1287, 0.3880, 0.1731, 0.0441],
    [0.0895, 0.0585, 0.0587, 0.0461, 0.2352, 0.3753, 0.1367],
    [0.2324, 0.0759, 0.0352, 0.0135, 0.0915, 0.2230, 0.3284],
]; // Markov chain rules for notes based off of mono-midi-transposition-dataset

const OCTAVES: [i8; 3] = [-12, 0, 12]; // Semitones shift for an octave
const OCTAVE_MARKOV: [[f32; 3]; 3] = [
    [0.7f32, 0.3f32, 0.0f32],
    [0.2f32, 0.6f32, 0.2f32],
    [0.0f32, 0.3f32, 0.7f32],
]; // Arbitrarily determined

const DURATIONS: [u8; 4] = [1, 2, 4, 8];
const DURATION_MARKOV: [[f32; 4]; 4] = [
    [0.8065f32, 0.1581f32, 0.0307f32, 0.0047f32],
    [0.2546f32, 0.6458f32, 0.0874f32, 0.0122f32],
    [0.2233f32, 0.4628f32, 0.2864f32, 0.0275f32],
    [0.1376f32, 0.2615f32, 0.1796f32, 0.4213f32],
]; // Based off of mono-midi-transposition-dataset

#[derive(Clone, Debug, serde::Serialize)]
pub struct Note {
    pub pitch: i8,
    pub velocity: f32,
    pub duration: u8,
}

struct NoiseRng {
    x: u32,
    y: i32,
}

impl NoiseRng {
    fn new(start: u32, seed: i32) -> Self {
        Self { x: start, y: seed }
    }

    fn sample_next(&mut self) -> f32 {
        let x = self.x as f32;
        let y = self.y as f32 / 256.;
        // Seed chosen by keyboard mash, guaranteed to be random
        let res = OpenSimplex2.sample_with_seed([x, y], 207482365);
        self.x += 1;
        (res + 1.) / 2.
    }

    fn sample_range(&mut self, range: Range<f32>) -> f32 {
        self.sample_next() * (range.end - range.start) + range.start
    }
}

struct MarkovDistribution<T, const N: usize> {
    weights: [[f32; N]; N],
    slice: [T; N],
    last_idx: usize,
}

impl<T: Copy, const N: usize> MarkovDistribution<T, N> {
    fn new(slice: [T; N], mut weights: [[f32; N]; N]) -> Self {
        for indiv_weight in &mut weights {
            let mut acc = 0.;
            *indiv_weight = indiv_weight.map(|x| {
                acc += x;
                acc
            });
        }

        Self {
            weights,
            slice,
            last_idx: N / 2,
        }
    }

    fn sample(&mut self, rng: &mut NoiseRng) -> T {
        let row = &self.weights[self.last_idx];

        let probability = rng.sample_next();

        let next = row.iter().position(|&x| probability < x).unwrap();

        self.last_idx = next;

        self.slice[next]
    }
}

pub struct NoteGenerator {
    rng: NoiseRng,
    pitch: MarkovDistribution<i8, 7>,
    octave: MarkovDistribution<i8, 3>,
    duration: MarkovDistribution<u8, 4>,

    velocity: f32,
    deltav: f32,
    key_offset: i8,
}

impl NoteGenerator {
    pub fn new(start: u32, seed: i32) -> Self {
        let mut rng = NoiseRng::new(start, seed);
        let key_offset = rng.sample_range(-6.0..6.0) as i8;

        Self {
            rng,
            pitch: MarkovDistribution::new(SCALE, MARKOV_SCALE),
            octave: MarkovDistribution::new(OCTAVES, OCTAVE_MARKOV),
            duration: MarkovDistribution::new(DURATIONS, DURATION_MARKOV),
            velocity: 0.5,
            deltav: 0.0,
            key_offset,
        }
    }

    fn next_velocity(&mut self) -> f32 {
        self.deltav = if self.deltav.is_sign_negative() {
            self.rng.sample_range(-0.03..0.1)
        } else {
            self.rng.sample_range(-0.1..0.03)
        };

        self.velocity += self.deltav;

        if self.velocity < 0.3 {
            self.velocity = 0.3;
            self.deltav = 1.0;
        } else if self.velocity > 0.8 {
            self.velocity = 0.8;
            self.deltav = -1.0;
        }

        self.velocity
    }

    fn next_pitch(&mut self) -> i8 {
        self.pitch.sample(&mut self.rng) + self.octave.sample(&mut self.rng) + self.key_offset
    }

    fn next_note(&mut self) -> Note {
        Note {
            pitch: self.next_pitch(),
            velocity: self.next_velocity(),
            duration: self.duration.sample(&mut self.rng),
        }
    }

    pub fn transpose(&mut self, semitones: i8) {
        self.key_offset += semitones;
    }
}

impl Iterator for NoteGenerator {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_note())
    }
}
