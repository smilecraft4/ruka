extern crate ffmpeg_next as ffmpeg;

use anyhow::Result;
use ffmpeg::{codec, ffi::AVFormatContext, filter, format, frame, media};
use std::{
    ffi::c_void,
    io::Read,
    path::{Path, PathBuf},
};

// TODO: Remove link
const PATH: &str = "examples/data/triples - girls' capitalism (official audio)";

const OUT_EXTENSIONS: &str = ".wav";

fn filter(
    spec: &str,
    decoder: &codec::decoder::Audio,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg::Error> {
    let mut filter = filter::Graph::new();

    let args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    filter.add(&filter::find("abuffer").unwrap(), "in", &args)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

    {
        let mut out = filter.get("out").unwrap();

        out.set_sample_format(encoder.format());
        out.set_channel_layout(encoder.channel_layout());
        out.set_sample_rate(encoder.rate());
    }

    filter.output("in", 0)?.input("out", 0)?.parse(spec)?;
    filter.validate()?;

    println!("{}", filter.dump());

    if let Some(codec) = encoder.codec() {
        if !codec
            .capabilities()
            .contains(ffmpeg::codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
        {
            filter
                .get("out")
                .unwrap()
                .sink()
                .set_frame_size(encoder.frame_size());
        }
    }

    Ok(filter)
}

struct Transcoder {
    stream: usize,
    filter: filter::Graph,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
    in_time_base: ffmpeg::Rational,
    out_time_base: ffmpeg::Rational,
}

fn transcoder<P: AsRef<Path>>(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    path: &P,
    filter_spec: &str,
) -> Result<Transcoder, ffmpeg::Error> {
    let input = ictx
        .streams()
        .best(media::Type::Audio)
        .expect("could not find best audio stream");
    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
    let mut decoder = context.decoder().audio()?;
    let codec = ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio))
        .expect("failed to find encoder")
        .audio()?;
    let global = octx
        .format()
        .flags()
        .contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER);

    decoder.set_parameters(input.parameters())?;

    let mut output = octx.add_stream(codec)?;
    let context = ffmpeg::codec::context::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(ffmpeg::channel_layout::ChannelLayout::STEREO);

    if global {
        encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
    }

    println!("rate {}", decoder.rate());
    println!("bit_rate {}", decoder.bit_rate());
    println!("max_bit_rate {}", decoder.max_bit_rate());

    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channel_layout(channel_layout);
    encoder.set_channels(channel_layout.channels());
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(decoder.bit_rate());
    encoder.set_max_bit_rate(decoder.max_bit_rate());

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    let filter = filter(filter_spec, &decoder, &encoder)?;

    let in_time_base = decoder.time_base();
    let out_time_base = output.time_base();

    Ok(Transcoder {
        stream: input.index(),
        filter,
        decoder,
        encoder,
        in_time_base,
        out_time_base,
    })
}

impl Transcoder {
    fn send_frame_to_encoder(&mut self, frame: &ffmpeg::Frame) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(&mut self, octx: &mut format::context::Output) {
        let mut encoded = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            encoded.rescale_ts(self.in_time_base, self.out_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }

    fn add_frame_to_filter(&mut self, frame: &ffmpeg::Frame) {
        self.filter.get("in").unwrap().source().add(frame).unwrap();
    }

    fn flush_filter(&mut self) {
        self.filter.get("in").unwrap().source().flush().unwrap();
    }

    fn get_and_process_filtered_frames(&mut self, octx: &mut format::context::Output) {
        let mut filtered = frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered)
            .is_ok()
        {
            self.send_frame_to_encoder(&filtered);
            self.receive_and_process_encoded_packets(octx);
        }
    }

    fn send_packet_to_decoder(&mut self, packet: &ffmpeg::Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(&mut self, octx: &mut format::context::Output) {
        let mut decoded = frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);
            self.add_frame_to_filter(&decoded);
            self.get_and_process_filtered_frames(octx);
        }
    }
}

pub fn input_buffer(buffer: &mut Vec<u8>) -> Result<format::context::Input, ffmpeg::Error> {
    unsafe {
        let size = buffer.len() * std::mem::size_of::<u8>();
        println!("buffer size {size}");

        let buffer = ffmpeg::ffi::av_memdup(buffer.as_ptr() as *const c_void, size);

        let ctx: *mut ffmpeg::ffi::AVIOContext = ffmpeg::ffi::avio_alloc_context(
            buffer as *mut u8,
            size as i32,
            0,
            std::ptr::null_mut(),
            None,
            None,
            None,
        );

        println!("Checkpoint 2");

        let mut format_ctx: *mut AVFormatContext = ffmpeg::ffi::avformat_alloc_context();
        (*format_ctx).pb = ctx;

        dbg!(*format_ctx);

        println!("Checkpoint 3");

        match ffmpeg::ffi::avformat_open_input(
            &mut format_ctx,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            0 => match ffmpeg::ffi::avformat_find_stream_info(format_ctx, std::ptr::null_mut()) {
                r if r >= 0 => Ok(format::context::Input::wrap(format_ctx)),
                e => {
                    ffmpeg::ffi::avformat_close_input(&mut format_ctx);

                    println!("Error 1");
                    Err(ffmpeg::Error::from(e))
                }
            },

            e => {
                println!("Error 2");
                Err(ffmpeg::Error::from(e))
            }
        }
    }
}

fn read_file_to_buffer(file_path: &str) -> Result<Vec<u8>> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn main() -> Result<()> {
    // User inputs
    let input_path = PathBuf::from(format!("{PATH}.weba"));
    let output_path = PathBuf::from(format!("{PATH}{OUT_EXTENSIONS}"));
    let filter = "anull".to_owned();

    println!("Start");

    let mut buffer = read_file_to_buffer(&input_path.to_str().unwrap())?;

    println!("Checkpoint 1");

    let mut ictx = input_buffer(&mut buffer)?;

    println!("Checkpoint 4");

    let mut octx = ffmpeg_next::format::output(&output_path)?;
    let mut transcoder = transcoder(&mut ictx, &mut octx, &output_path, &filter)?;

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header()?;

    for (stream, mut packet) in ictx.packets() {
        if stream.index() == transcoder.stream {
            packet.rescale_ts(stream.time_base(), transcoder.in_time_base);
            transcoder.send_packet_to_decoder(&packet);
            transcoder.receive_and_process_decoded_frames(&mut octx);
        }
    }

    transcoder.send_eof_to_decoder();
    transcoder.receive_and_process_decoded_frames(&mut octx);

    transcoder.flush_filter();
    transcoder.get_and_process_filtered_frames(&mut octx);

    transcoder.send_eof_to_encoder();
    transcoder.receive_and_process_encoded_packets(&mut octx);

    octx.write_trailer().unwrap();

    Ok(())
}
