# Overview

Example of generating binaural sine waves using the STM32G0's DAC and DMA in Rust. It generates two sine waves at 440Hz and 294Hz.

This seems like a textbook example of why to use DMA, because the chip's top speed of 64MHz was too slow to generate two audio channels using timer interrupts. I probably could have made it work by cleaning the code up, moving the interrupts to RAM, etc. But by using the timers to trigger DMA transfers instead of interrupts, the CPU is freed up and the program can run fine at 16MHz.

Pins A4 and A5 are used for the two DAC channels; I think they're the only two DAC pins available on the STM32G071, but they have 12-bit resolution which should be good enough for starting to learn about audio. Pin C6 is used as a 'heartbeat' LED output.

You'll probably need an audio amplifier to drive a speaker. Connect PA4 and PA5 to the amp's L/R channels, not directly to a speaker. Companies like Adafruit and Sparkfun sell small breadboard-friendly amps with solid documentation, in my personal opinion.

The same 'DMA triggered by hardware timers' recipe should also work with peripherals like SPI, UART, I2C, etc. DMA is probably also useful for shuttling framebuffers to displays and things like that.

# Peripheral Access Crate

This example uses a PAC which was auto-generated from the STM32G0 SVD files distributed by ST, but I've also been playing with adding descriptive names to register bitfields. This is sort of a reference repository, so if you build it with the STM32G0 PAC generated by `svd2rust`, you'll need to change some of the GPIO mode register definitions. Like, `moder6().output()` would need to become `moder6().bits(1)`.
