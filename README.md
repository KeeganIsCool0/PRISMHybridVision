# <img width="1100" height="350" alt="prismlogo" src="https://github.com/user-attachments/assets/4e40071b-a3a9-403c-a892-f968315ee6f5" />

<div align="center">
  
[![CC BY-SA 4.0][cc-by-sa-shield]][cc-by-sa] 
[![License: CERN-OHL-S v2](https://img.shields.io/badge/License-CERN--OHL--S%20v2-orange.svg?logo=circuitverse&logoColor=white)](https://ohwr.org/cern_ohl_s_v2.txt)
![C++](https://img.shields.io/badge/C%2B%2B-17-blue.svg?logo=c%2B%2B&logoColor=white) 
![Python](https://img.shields.io/badge/python-3.11-blue.svg?logo=python&logoColor=white) ![License](https://img.shields.io/badge/license-GPLv3-blue.svg?logo=gnu&logoColor=white)
![GitHub Repo stars](https://img.shields.io/github/stars/KeeganIsCool0/PRISMHybridVision) 
![GitHub forks](https://img.shields.io/github/forks/KeeganIsCool0/PRISMHybridVision) ![GitHub release](https://img.shields.io/github/v/release/KeeganIsCool0/PRISMHybridVision) 

</div>

[cc-by-sa]: http://creativecommons.org/licenses/by-sa/4.0/
[cc-by-sa-image]: https://licensebuttons.net/l/by-sa/4.0/88x31.png
[cc-by-sa-shield]: https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg

<div align="center">

PRISM Hybrid Vision is a digital-analog recording console that combines digital workflows with true analog sound. 

</div>

<div align="center">

[Documentation](./Docs) · [Firmware](./Firmware) · [Hardware](./Hardware)
  
</div>

<div align="center">

  [![Donate](https://img.shields.io/badge/Donate-GoFundMe-00B964?style=for-the-badge&logo=gofundme&logoColor=white)](https://gofund.me/79449fde0)

</div>

> This project is under active development. Bugs are expected.

## Overview

<img width="1240" height="691" alt="image" src="https://github.com/user-attachments/assets/37b4c516-2eff-4e29-808a-87cdaa8106a7" />

### Description

  The PRISM Hybrid Vision is a hybrid digital/analog immersive sound mixing and mastering console. The console is designed for DAW workflows and professional studio use. The console also has a modular architecture, so you can replace modules with new ones without replacing the entire console. The console also has full DAW automation without compromising on true analog sound. The console also has switchable preamp, conversion, and output styles. It can select from OLD, OLD-NEW, and NEW. Each channel has a 5 band and 10 band EQ, the 5 band is for broader selection of frequencies, the 10 band has a more sophisticated selection of frequencies to choose from. There are two compressors per channel. One VCA and one Opto compressor chained together. These two compressors create the best duo per channel for excellent sound. Each channel comes with 3 Stereo AUX sends. Each channel also comes with MUTE, SOLO, RECORD, and CUT buttons, which are fully automated from the center console.

## Why PRISM?

* PRISM is modular: No need to replace the console! Each channel module can be replaced with anew compatible one!
* No compromises: This console combines true analog warmth with digital control, automation, and DAW integration.
* Instead of buying different types of processors, you can select them purely on the console
* Digital routing and processing makes studio workflow faster using automation and recall.

## Key Features

* Hybrid analog paths
* Digital recall
* Motorized faders
* Modular Cards
* Switchable preamp color and style
* 2 Compressors per channel
* MADI Networking
* Touchscreen control
* Expandable architecture

## General Specifications

* 48 Input channels
- 48 Mic preamps
* 48 Line Inputs
- 16 Stereo Returns
* 16 AUX Sends
- 8 Sub-Groups
* Stereo + Monitor Outputs
- 44.1-192 kHz Sample Rates
* Bit Depth: 24-Bit
- MADI Digital Routing Interface
* Full Recall
- 100 mm motorized faders per channel

## Sponsorship Inquiries

- If you have questions, please fill out the contact forum @ https://tally.so/r/wzlvW8
- You can find more information here: 
- When clicking the .pdf link, you will be redirected to a new branch, go back to the main branch by selecting the branches at the top.

## Why Sponsor now?

### Lowest cost, Highest influence
- Before fabrication, your input influences design decisions. Like get input on features you may want and component choices.

### Cheapest dollar-for-impact
- Sponsors can donate to the project from the beginning while contributing minimally.

### First-mover association
- Early sponsorship provides the opportunity to support PRISM at its most influential stage; help shape the project from documentation to prototyping.

### Money is needed at this stage
- Fabrication is the most expensive step. If sponsors invest later, it is less needed and less rewarded.

## What You Get*

* Brand Placement: Your logo or name will be featured on the PRISM console, documentation, and on the KV designs website.
* Public credit, Named as a partner/sponsor in the spec doc, repo, and any demo material.
* Early Access, First look at working prototypes, and input on features that affect your use case.

## About the Engineer
PRISM is designed and documented by a solo audio hardware designer. What started as a fascination with electronics has grown into building a full console combining analog processing, digital control, automation, and immersive audio architecture. This project has progressed from concept to a project with technical documentation, and a prototype test plan completed. The next step is bringing Prototype 1A into the real world.

## Firmware

A C++20 production panner that renders a mono source to a **17-channel 9.2.6 speaker bed** (9 bed speakers, two LFE feeds, and six height speakers). It provides a compact producer UI for azimuth, elevation, distance, and spread, plus per-output trim and two shelving filters for room correction.

This project sends PCM through PipeWire. It does **not** create Dolby Atmos ADM metadata or a Dolby-encoded master; Dolby encoding and certified rendering require Dolby's licensed tooling. Route the PipeWire node to a correctly configured 17-channel / 9.2.6 endpoint or monitoring graph.

### Build

Fedora/RHEL:

```sh
sudo dnf install cmake gcc-c++ pipewire-devel
cmake -S . -B build
cmake --build build
ctest --test-dir build --output-on-failure
```

Run it within a logged-in PipeWire session:

```sh
./build/atmos-panner
```

The source is currently a 440 Hz reference tone. Replace the tone generator in `PipeWireOutput.cpp` with decoded input audio (or add an input stream) for program material.

### Producer controls

At the spatial-panning prompt:

```text
pos <azimuth> <elevation> <distance>
spread <degrees>
trim <channel 0-16> <dB>
room <channel 0-16> <low-shelf dB> <high-shelf dB>
quit
```

Azimuth is -180° (rear left) to +180° (rear right); elevation is -30° to +90°. Channel indices follow the displayed `FL, FC, FR, WL, WR, SL, SR, BL, BR, LFE1, LFE2, TFL, TFR, TML, TMR, TBL, TBR` order.

## Test the Channel Strip

Go to https://github.com/KeeganIsCool0/PRISM_ChannelStrip to test the channel strip on your computer!†

## Channel Model
<img width="595" height="841" alt="image" src="https://github.com/user-attachments/assets/79c7f222-74c7-4211-ae90-7f256c7b518e" />

## Center Console

<img width="1240" height="772" alt="image" src="https://github.com/user-attachments/assets/26712e4a-191a-48d1-bb43-06089ded5b41" />

## Designing Process

<img width="1920" height="1080" alt="Whiteboard" src="https://github.com/user-attachments/assets/e1de475b-02ea-4c5a-97b8-b2637a0bed23" />

## Development Status

### Engineering
- [x] System Architecture
- [x] Channel Strip Design
- [x] Modular Processing Architecture
- [x] Digital Control & Recall
- [x] Immersive Audio Architecture
- [x] Schematics
- [x] Simulation
- [x] Technical Specifications
- [x] 2D CAD
- [ ] Prototype Fabrication
- [ ] Prototype Testing
- [ ] Design Revision

### Documentation
- [x] Product Overview
- [x] Technical Specification
- [x] System Architecture
- [x] Test Plan
- [ ] Test Results

### Sponsorship
- [x] Executive Summary
- [x] Sponsor Presentation
- [x] Prototype Budget / BOM
- [x] Development Roadmap
- [ ] Sponsorship Outreach
- [ ] Sponsorship Agreements

### Production
- [ ] Manufacturing Validation
- [ ] Production Design
- [ ] Manufacturing

## Comparison

| Console Type             | Analog Character | Digital Recall   | Full Automation  | Modular Expansion |
|--------------------------|------------------|------------------|------------------|-------------------|
| No Console               | :x:              | :x:              | :x:              | :x:               |
| Vintage Analog Consoles  |:white_check_mark:| :x:              | :x:              | :x:               |
| Fully Digital Consoles   | :x:              |:white_check_mark:|:white_check_mark:| Limited           |
| Hybrid Consoles          | Partial          |:white_check_mark:|Partial           | :x:               |
| PRISM Hybrid Vision      |:white_check_mark:|:white_check_mark:|:white_check_mark:|:white_check_mark: |

## Development

To get schematics and other development info please fill the contact form @ https://tally.so/r/wzlvW8

Thank you for being interested in this project!

## Credits

This project would not be possible without SSL, Neve, and API. Thank you for giving me the inspiration for this project.

## Sponsors

You can click the "Sponsor" button or go to the GoFundMe here: https://gofund.me/b56c8fe3b 

<a href="https://www.buymeacoffee.com/keeganstudio" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>
[![Donate](https://img.shields.io/badge/Donate-GoFundMe-00B964?style=for-the-badge&logo=gofundme&logoColor=white)](https://gofund.me/79449fde0)

PRISM Hybrid Vision is currently seeking sponsors and technical partners to support prototype fabrication, component acquisition, testing, and development.

## LICENSE

- **Code**: Licensed under the [GPLv3 LICENCE](LICENSE)
- **Documentation**: Licensed under [CC BY-SA 4.0](./Docs/LICENCE-DOCS)
- **Schematics, PCB Designs, CAD, and hardware**: Licensed under [CERN-OHL-S](./Docs/CAD/LICENCE-CAD)

## Contributors & Sponsors Hall Of Fame

[![Contributors](https://contrib.rocks/image?repo=KeeganIsCool0/PRISMHybridVision)](https://github.com/KeeganIsCool0/PRISMHybridVision/graphs/contributors)

PRISM Hybrid Vision is currently seeking sponsors and technical partners to support prototype fabrication, component acquisition, testing, and development.


## NOTICE
- *Depends on sponsor tier.
- † Application does not emulate true analog sound. 

<div align="center">

[![CC BY-SA 4.0][cc-by-sa-image]][cc-by-sa]

[cc-by-sa]: http://creativecommons.org/licenses/by-sa/4.0/
[cc-by-sa-image]: https://licensebuttons.net/l/by-sa/4.0/88x31.png
[cc-by-sa-shield]: https://img.shields.io/badge/License-CC%20BY--SA%204.0-lightgrey.svg
  
</div>


