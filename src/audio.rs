use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub(crate) struct AudioData {
    pub(crate) samples: Vec<f32>,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
}

pub(crate) fn load_audio(filename: &str) -> Result<AudioData, String> {
    let file = File::open(filename).map_err(|e| format!("{}: {}", filename, e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("{}: unsupported format ({})", filename, e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| format!("{}: no audio track found", filename))?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| format!("{}: unknown sample rate", filename))?;
    let channels = track
        .codec_params
        .channels
        .ok_or_else(|| format!("{}: unknown channel count", filename))?
        .count() as u16;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("{}: failed to create decoder ({})", filename, e))?;

    let track_id = track.id;
    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("{}: error reading packet ({})", filename, e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: decode error: {}", e);
                continue;
            }
        };

        let spec = *decoded.spec();
        let num_channels = spec.channels.count();

        let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);

        for chunk in sample_buf.samples().chunks(num_channels) {
            let mono: f32 = chunk.iter().sum::<f32>() / num_channels as f32;
            samples.push(mono);
        }
    }

    Ok(AudioData {
        samples,
        sample_rate,
        channels,
    })
}
