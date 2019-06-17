#![no_std]
#![no_main]

// Halt when the program panics.
extern crate panic_halt;

// Generic ARM Cortex-M includes.
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m_rt::entry;
// Chip-specific PAC include.
use stm32_pac::stm32g0x1 as stm32;

const SINE_SAMPLES : usize = 32;
const SINE_WAVE : [ u16; SINE_SAMPLES ] = [
  2048, 2447, 2831, 3185,
  3495, 3750, 3939, 4056,
  4095, 4056, 3939, 3750,
  3495, 3185, 2831, 2447,
  2048, 1649, 1265, 911,
  601,  346,  157,  40,
  0,    40,   157,  346,
  601,  911,  1265, 1649
];

// System clock doesn't need to change from 16MHz HSI yet.
static mut SYS_CLK_HZ : u32 = 16_000_000;

#[entry]
fn main() -> ! {
  // Checkout ARM Cortex-M peripheral singleton..
  let cm_p = cortex_m::Peripherals::take().unwrap();
  let mut syst = cm_p.SYST;
  // Checkout STM32 peripheral singleton.
  let p = stm32::Peripherals::take().unwrap();
  let rcc = p.RCC;
  let dac = p.DAC;
  let dma = p.DMA;
  let dmamux = p.DMAMUX;
  //let flash = p.FLASH;
  let gpioa = p.GPIOA;
  let gpioc = p.GPIOC;
  let tim6 = p.TIM6;
  let tim7 = p.TIM7;

  // (Optional) Initialize system clock to 64MHz.
  // Use the default HSI16 osc to drive the PLL.
  /*
  unsafe {
    // Set 2 Flash wait states.
    flash.acr.modify( |_r,w| w.latency().bits( 0b010 ) );
    // Configure PLL settings.
    // freq = ( 16MHz * ( N / M ) ) / R
    // For 64MHz, R = 2, M = 1, N = 8.
    rcc.pllsyscfgr.modify( |_r,w| w.pllr().bits( 0b001 )
                                   .pllren().set_bit()
                                   .plln().bits( 0b0001000 )
                                   .pllm().bits( 0b000 )
                                   .pllsrc().bits( 0b10 ) );
    // Enable and select PLL.
    rcc.cr.modify( |_r,w| w.pllon().set_bit() );
    while !rcc.cr.read().pllrdy().bit_is_set() {};
    rcc.cfgr.modify( |_r,w| w.sw().bits( 0b010 ) );
    while rcc.cfgr.read().sws().bits() != 0b010 {};
    // Update system clock value.
    SYS_CLK_HZ = 64_000_000;
  }
  */

  // Select audio tones.
  // Sine wave freq = ( timer trigger freq ) / ( # samples )
  // A4 = 440Hz, D4 ~= 293.67Hz
  // Each DAC will move on to the next sample every time
  // that the connected timer ticks over.
  let l_tim_freq : u32 = 440 * SINE_SAMPLES as u32;
  let r_tim_freq : u32 = 294 * SINE_SAMPLES as u32;
  let l_tim_arr  : u16;
  let r_tim_arr  : u16;
  unsafe {
    l_tim_arr = ( SYS_CLK_HZ / l_tim_freq ) as u16;
    r_tim_arr = ( SYS_CLK_HZ / r_tim_freq ) as u16;
  }

  // Enable peripheral clocks.
  // GPIO ports A and C.
  rcc.iopenr.write( |w| w.iopaen().set_bit()
                         .iopcen().set_bit() );
  // Enable Digital-Analog Converter, Timers 6 & 7.
  rcc.apbenr1.write( |w| w.dac1en().set_bit()
                          .tim6en().set_bit()
                          .tim7en().set_bit() );
  // Enable Direct-Memory Access.
  rcc.ahbenr.write( |w| w.dmaen().set_bit() );

  // Setup GPIO pin C6 (LED) as push-pull output.
  gpioc.moder.modify( |_r,w| w.moder6().output() );
  gpioc.otyper.modify( |_r,w| w.ot6().push_pull() );
  // Setup GPIO pins A4 and A5 as DAC outputs.
  gpioa.moder.modify( |_r,w| w.moder4().analog()
                              .moder5().analog() );

  // Setup stereo audio tones on DAC1/2 using DMA1/2 with
  // TIM6/7 as request generation triggers.
  unsafe {
    // Setup DMA channels 0/1 to send data to DAC channels 1/2.
    // - Priority level: high (2)
    // - Memory/Peripheral request size: 16-bit (1)
    // - Cirular mode: Enabled.
    // - Increment memory ptr, do not increment periph ptr.
    // - Disable 'memory-to-memory' mode.
    // - Set 'memory -> peripheral' transfer direction.
    dma.ccr1.write( |w| w.pl().bits( 0b10 )
                         .msize().bits( 0b01 )
                         .psize().bits( 0b01 )
                         .circ().set_bit()
                         .minc().set_bit()
                         .pinc().clear_bit()
                         .mem2mem().clear_bit()
                         .dir().set_bit() );
    dma.ccr2.write( |w| w.pl().bits( 0b10 )
                         .msize().bits( 0b01 )
                         .psize().bits( 0b01 )
                         .circ().set_bit()
                         .minc().set_bit()
                         .pinc().clear_bit()
                         .mem2mem().clear_bit()
                         .dir().set_bit() );
    // Route DMA channels 1/2 to DAC channels 1/2.
    // (Only DMAMUX is 0-indexed, sorry that it's confusing.)
    dmamux.dmamux_c0cr.write( |w| w.dmareq_id().bits( 0x8 ) );
    dmamux.dmamux_c1cr.write( |w| w.dmareq_id().bits( 0x9 ) );
    // Set DMA transfer size and source/destination addresses.
    // TODO: Get register addresses from PAC values.
    let sin_wave_addr : u32 = &SINE_WAVE as *const [ u16; SINE_SAMPLES ] as u32;
    dma.cndtr1.write( |w| w.ndt().bits( SINE_SAMPLES as u16 ) );
    dma.cpar1.write( |w| w.pa().bits( 0x40007408 ) );
    dma.cmar1.write( |w| w.ma().bits( sin_wave_addr ) );
    dma.cndtr2.write( |w| w.ndt().bits( SINE_SAMPLES as u16 ) );
    dma.cpar2.write( |w| w.pa().bits( 0x40007414 ) );
    dma.cmar2.write( |w| w.ma().bits( sin_wave_addr ) );
    // Enable both DMA channels.
    dma.ccr1.modify( |_r,w| w.en().set_bit() );
    dma.ccr2.modify( |_r,w| w.en().set_bit() );

    // TIM6 (Left channel).
    tim6.psc.write( |w| w.psc().bits( 0 ) );
    tim6.arr.write( |w| w.arr().bits( l_tim_arr ) );
    tim6.cr2.write( |w| w.mms().bits( 0b10 ) );
    tim6.cr1.write( |w| w.cen().set_bit() );
    // TIM7 (Right channel).
    tim7.psc.write( |w| w.psc().bits( 0 ) );
    tim7.arr.write( |w| w.arr().bits( r_tim_arr ) );
    tim7.cr2.write( |w| w.mms().bits( 0b10 ) );
    tim7.cr1.write( |w| w.cen().set_bit() );

    // Enable both DAC channels, set up to trigger on TIM6/7.
    dac.dac_mcr.write( |w| w.mode1().bits( 0b00 )
                            .mode2().bits( 0b00 ) );
    dac.dac_cr.modify( |_r,w| w.en1().set_bit()
                               .en2().set_bit()
                               .dmaen1().set_bit()
                               .dmaen2().set_bit()
                               .tsel1().bits( 0x5 )
                               .tsel2().bits( 0x6 ) );
    // Delay ~1ms using SysTick to let the DAC wake up.
    syst.set_clock_source( SystClkSource::Core );
    syst.set_reload( SYS_CLK_HZ / 1000 );
    syst.clear_current();
    syst.enable_counter();
    while !syst.has_wrapped() {};
    syst.disable_counter();
    // Enable DAC triggers.
    dac.dac_cr.modify( |_r,w| w.ten1().set_bit()
                               .ten2().set_bit() );
  }

  // Set ~0.25s SysTick period. It's a 24-bit timer,
  // so @64MHz we can't make a simple 1-second tick.
  // (Or @16MHz, this is a 1-second tick.)
  syst.set_clock_source( SystClkSource::Core );
  syst.set_reload( 16_000_000 );
  // Restart the SysTick counter.
  syst.clear_current();
  syst.enable_counter();

  // Main loop.
  loop {
    // Toggle the LED every SysTick tick.
    while !syst.has_wrapped() {};
    gpioc.odr.write( |w| w.odr6().set_bit() );
    while !syst.has_wrapped() {};
    gpioc.odr.write( |w| w.odr6().clear_bit() );
  }
}
