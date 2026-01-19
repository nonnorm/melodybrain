# MelodyBrain – Hear the World’s Melody

Join the global music experiment! Each heartbeat you send contributes to a procedural song generated from the combined seeds of every connected user.

### A satirical response to [Hypermind](https://github.com/lklynet/hypermind)

**MelodyBrain** is an open-source, completely centralized, peer-to-server deployment counter and global music generator.

It solves the non-critical but extremely fun challenge of:
1. Knowing how many people are running this lightweight, fast, secure, and containerless binary.
2. Procedurally generating music based on a weighted average of computer seeds worldwide. Contribute to the ~~earswarm~~ melody!

## What is this?
In all honesty, it's a proof that 'serverless' and 'distributed' doesn't mean better. This is one UDP server, hosted on one VPS to serve the world. Can it scale to tens of thousands of people? Let's find out together.

Containers? Systemd? Run it with a plain bash script that installs the binary, starts and stops the daemon, and gets rid of it if you've had your fun.

Also it's so much cooler than Hypermind because you're actually contributing to something while making numbers go up.

## How it works
1. **Connect** to the UDP server
2. **Send heartbeats** every so often
3. **Receive the global seed** and hear the melody of the world *and* individual countries

> Your actions literally shape the music!

## How to install
```sh
curl -sSfLO 'https://raw.githubusercontent.com/nonnorm/melodybrain/refs/heads/main/melodybrain.sh'
chmod +x ./melodybrain.sh
./melodybrain.sh install
./melodybrain.sh start # OR ~/.melodybrain/melodybrain to not run as a daemon
```

## FAQ:
**Q: Does it crypto mine?** A: No, but it can always be added later if you want to waste some more processing power.

**Q: Does it store data?** A: The only running count stored on the server is the **global and per-country seeds**. It's a weighted average that everybody contributes to. Aside from that, your IP address will be stored for as long as you keep sending heartbeats so that we have an acurate number of connections.

**Q: Why is it not distributed?** A: Doing cool distributed hash table stuff is... cool! But it makes everything about this harder. It involves complicated algorithms, NAT hole-punching, *and you still need bootstrap nodes to get it started*. What if your bootstrap node was just your source of truth to begin with? Oh look, we've reinvented the internet.

**Q: Why did you make this? Are you trying to steal Hypermind's thunder?** A: I was inspired by Hypermind, and I was originally going to go in the same direction as it. But it was just so... complicated to do. So I switched tactics. Consider this the Anti-Hypermind, proving that simplicity can triumph over "cool." But seriously, if you want to run Hypermind over this, or just run both, knock yourself out. :)

## Supported Platforms
- **macOS** – Intel (x86_64) & Apple Silicon (arm64)
- **Linux** – x86_64, ARM 64-bit (aarch64/arm64), ARM 32-bit (armv7, armv6)

## Upcoming Features:
- Perhaps some level of distribution. A few trusted server nodes will be more resilient than one.
- Windows support
- Adding chaos? Randomly change the noise generation seed or reset each seed once a certain number of people contribute to it?



