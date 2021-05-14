use tuix::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::thread;

mod controller;
mod message;

static THEME: &'static str = include_str!("theme.css");

fn main() {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(1024);

    thread::spawn(move || {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                run::<f32>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::I16 => {
                run::<i16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }

            cpal::SampleFormat::U16 => {
                run::<u16>(&device, &config.into(), command_receiver.clone()).unwrap();
            }
        }
    });

    // Create a gui application on the main thread
    let app = Application::new(|win_desc, state, window|{
        
        state.style.parse_theme(THEME);

        controller::Controller::new(command_sender.clone()).build(state, window, |builder| builder);

        win_desc.with_title("Audio Synth").with_inner_size(200, 120)
    
    });

    app.run();
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    command_receiver: crossbeam_channel::Receiver<dyn Message>,
) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    // Get the sample rate and channels number from the config
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    // Define some variables we need for a simple oscillator
    let mut phi = 0.0f32;
    let mut frequency = 440.0f32;
    let mut amplitude = 1.0;
    let mut note = 0.0;

    // Build an output stream
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            // A frame is a buffer of samples for all channels. So for 2 channels it's 2 samples.
            for frame in data.chunks_mut(channels) {
                // Try to receive a message from the gui thread
                while let Ok(command) = command_receiver.try_recv() {
                    match command {
                        Message::Note(val) => {
                            note = val;
                        }

                        Message::Amplitude(val) => {
                            amplitude = val;
                        }

                        Message::Frequency(val) => {
                            frequency = val;
                        }
                    }
                }

                // This creates a 'phase clock' which varies between 0.0 and 1.0 with a rate of frequency
                phi = (phi + (frequency / sample_rate)).fract();

                // Generate a sine wave signal
                let make_noise =
                    |phi: f32| -> f32 { amplitude * note * (2.0f32 * 3.141592f32 * phi).sin() };

                // Convert the make_noise output into a sample
                let value: T = cpal::Sample::from::<f32>(&make_noise(phi));

                // Assign this sample to all channels in the frame
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn,
    )?;

    // Play the stream
    stream.play()?;

    // Park the thread so out noise plays continuously until the app is closed
    std::thread::park();

    Ok(())
}
