extern crate anyhow;
extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;

struct Saw {
    frequency: f32,
    count: i32,
    val: f32,
}
  
impl Saw {
  #[inline]
  fn set_frequency(&mut self, freq: &f32) {
    self.frequency = *freq;
  }
  #[inline]
  fn next_sample(&mut self, sample_rate: f32) -> f32 {
    if self.count >= (sample_rate / self.frequency) as i32 {
      self.count = 0;
    } else {
      self.count += 1;
    }
  
      
    if self.count == 0 {
      self.val = 1.0;
    } else {
      self.val -= 1.0 / (sample_rate / self.frequency);
    }

    self.val - 0.5
  }
}

// A synth voice. TODO add amplitude envelope
struct Voice {
    saw: Saw,
    state: usize,
    frequency: f32
}

impl Voice {
    fn note_on(&mut self) {
        self.state = 1
    }
    fn note_off(&mut self) {
        self.state = 0
    }
    fn set_freq(&mut self, freq: &f32) {
        self.frequency = *freq;
        self.saw.set_frequency(freq);
    }
}

struct Poly {
    sample_rate: f32,
    freq: f32,
    voices: Vec<Voice>,
    voice_count: usize
}

impl Poly {
    fn next_sample(&mut self) -> f32 {
        let mut out: f32 = 0.0;
        for voice in self.voices.iter_mut() {
            out += voice.saw.next_sample(self.sample_rate) * (voice.state as f32);
        }
        out
    }
    fn new_note(&mut self, frequency: &f32) {
        for voice in self.voices.iter_mut() {
            if voice.state == 0 {
                voice.set_freq(frequency);
                voice.note_on();
                return
            }
        }
        self.voices[self.voice_count - 1].note_off();
        self.voices[self.voice_count - 1].set_freq(frequency);
        self.voices[self.voice_count - 1].note_on();
        return
    }
}

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
    let mut children = vec![];

    children.push(thread::spawn( move ||  {
    #[cfg(all(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"), feature = "jack"))]
    // Manually check for flags. Can be passed through cargo with -- e.g.
    // cargo run --release --example beep --features jack -- --jack
    let host = if std::env::args()
        .collect::<String>()
        .contains(&String::from("--jack"))
    {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
                "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    } else {
        cpal::default_host()
    };

    #[cfg(any(
        not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
        not(feature = "jack")
    ))]
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    static mut saw: Saw = Saw {
        frequency: 110.0,
        count: 0,
        val: 0.0
    };

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), &mut saw).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), &mut saw).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), &mut saw).unwrap(),
    };}));
    
    println!("Thank you for choosing spicy software & co!");

    loop {}
    }

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, saw: &'static mut Saw) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;


    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _ : &cpal::OutputCallbackInfo| {
            write_data(data, channels, saw)
        },
        err_fn,
    )?;
    stream.play()?;

    loop {}

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, saw: &'static mut Saw)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&(saw.next_sample(44_100.0)));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}