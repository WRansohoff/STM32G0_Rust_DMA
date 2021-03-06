// Build script to copy memory segments into the default linker script.
// From the Rust embedded working group's ARM Cortex-M quickstart example.
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
  macro_rules! ld_mem { () => ( "ld/stm32g071xb.x" ) };

  // Put the linker script somewhere the linker can find it
  let out = &PathBuf::from( env::var_os( "OUT_DIR" ).unwrap() );
  File::create( out.join( "memory.x" ) )
      .unwrap()
      .write_all( include_bytes!( ld_mem!() ) )
      .unwrap();
  println!( "cargo:rustc-link-search={}", out.display() );

  // Only re-run the build script when memory.x is changed,
  // instead of when any part of the source code changes.
  //println!("cargo:rerun-if-changed={}", ld_mem);
}
