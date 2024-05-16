# neoVI MIC2 Application Programming Interfaces
https://github.com/intrepidcs/mic2/


![neoVI MIC2 Picture](../../neoVI-MIC-2.png)

[![crates.io](https://img.shields.io/crates/v/neovi-mic.svg)](https://crates.io/crates/mic2)
[![docs.rs](https://docs.rs/mic2/badge.svg)](https://docs.rs/mic2/)
<!-- [![CI](https://github.com/intrepidcs/mic2/workflows/CI/badge.svg)](https://github.com/intrepidcs/mic2/actions) -->

## **Description**

neoVI MIC 2 is a handheld pendant accessory with USB Trigger, GPS and microphone sold by Intrepid Control System, Inc.

## Installation

```cargo add mic2```

## **Examples**

See [Examples](https://github.com/intrepidcs/mic2/examples/rust/) on the github page for examples.


```rust
use mic2::find_neovi_mics;
use mic2::types::Result;

fn main() -> Result<()>{
    println!("Finding neovi MIC2 devices...");
    let devices = find_neovi_mics()?;

    println!("Found {} device(s)", devices.len());
    for device in devices {
        println!("{device:#?}");
    }

    Ok(())
}
```

## License
```
The MIT License

Copyright (c) <Intrepid Control Systems, Inc.>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```