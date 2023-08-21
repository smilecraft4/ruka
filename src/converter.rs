use std::{collections::HashMap, path::PathBuf};

use ffmpeg_next::{
    channel_layout, codec, encoder,
    ffi::{self, AVFormatContext},
    format, media,
};

use crate::error::Result;

pub fn convert_to_mp3(
    audio_source: &mut Vec<u8>,
    path: PathBuf,
    image: Vec<u8>,
    metadata: HashMap<String, String>,
) -> Result<()> {
    ffmpeg_next::init()?;

    // inputs
    let mut audio_input = input_buffer(audio_source)?;
    let mut audio_output = format::output(&path)?;

    // demuxing     avformat
    let input_stream = audio_input
        .streams()
        .best(media::Type::Audio)
        .expect("failed to find audio stream");

    let input_index = input_stream.index();

    // decode       avcodec
    let context = codec::Context::from_parameters(input_stream.parameters())?;
    let mut decoder = context.decoder().audio()?;

    // perform the conversion ?
    let codec = encoder::find(audio_output.format().codec(&path, media::Type::Audio))
        .expect("could not find codec for format")
        .audio()?;
    let global = audio_output
        .format()
        .flags()
        .contains(format::Flags::GLOBAL_HEADER);

    decoder.set_parameters(input_stream.parameters())?;

    // encode       avcodec
    let mut output = audio_output.add_stream(codec)?;
    let context = codec::Context::from_parameters(output.parameters())?;
    let mut encoder = context.encoder().audio()?;

    let channel_layout = codec
        .channel_layouts()
        .map(|cls| cls.best(decoder.channel_layout().channels()))
        .unwrap_or(channel_layout::ChannelLayout::STEREO);

    if global {
        encoder.set_flags(codec::flag::Flags::GLOBAL_HEADER);
    }
    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channel_layout(channel_layout);
    encoder.set_channels(channel_layout.channels());
    encoder.set_format(
        codec
            .formats()
            .expect("unkwown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(decoder.bit_rate());
    encoder.set_max_bit_rate(decoder.max_bit_rate());

    encoder.set_time_base((1, decoder.rate() as i32));
    output.set_time_base((1, decoder.rate() as i32));

    let mut encoder = encoder.open_as(codec)?;
    output.set_parameters(&encoder);

    let in_time_base = decoder.time_base();
    let out_time_base = output.time_base();

    audio_output.set_metadata(ffmpeg_next::Dictionary::from_iter(metadata.into_iter()));
    audio_output.write_header()?;

    for (stream, mut packet) in audio_input.packets() {
        if stream.index() == input_index {
            packet.rescale_ts(stream.time_base(), in_time_base);
            decoder.send_packet(&packet).unwrap();

            receive_and_process_decoded_frame(
                &mut decoder,
                &mut encoder,
                in_time_base,
                out_time_base,
                &mut audio_output,
            )?;
        }
    }

    decoder.send_eof()?;
    receive_and_process_decoded_frame(
        &mut decoder,
        &mut encoder,
        in_time_base,
        out_time_base,
        &mut audio_output,
    )?;

    encoder.send_eof()?;
    receive_and_process_encoded_packets(
        &mut encoder,
        in_time_base,
        out_time_base,
        &mut audio_output,
    )?;

    audio_output.write_trailer()?;

    Ok(())
}

fn receive_and_process_decoded_frame(
    decoder: &mut ffmpeg_next::decoder::Audio,
    encoder: &mut encoder::audio::Audio,
    in_time_base: ffmpeg_next::Rational,
    out_time_base: ffmpeg_next::Rational,
    audio_output: &mut format::context::Output,
) -> Result<()> {
    let mut decoded = ffmpeg_next::frame::Audio::empty();
    while decoder.receive_frame(&mut decoded).is_ok() {
        let timestamp = decoded.timestamp();
        decoded.set_pts(timestamp);
        encoder.send_frame(&decoded)?;

        receive_and_process_encoded_packets(encoder, in_time_base, out_time_base, audio_output)?;
    }

    Ok(())
}

fn receive_and_process_encoded_packets(
    encoder: &mut encoder::audio::Audio,
    in_time_base: ffmpeg_next::Rational,
    out_time_base: ffmpeg_next::Rational,
    audio_output: &mut format::context::Output,
) -> Result<()> {
    let mut encoded = ffmpeg_next::Packet::empty();
    while encoder.receive_packet(&mut encoded).is_ok() {
        encoded.set_stream(0);
        encoded.rescale_ts(in_time_base, out_time_base);
        encoded.write_interleaved(audio_output)?;
    }
    Ok(())
}

fn input_buffer(
    buffer: &mut Vec<u8>,
) -> core::result::Result<format::context::Input, ffmpeg_next::Error> {
    unsafe {
        let size = buffer.len() * std::mem::size_of::<u8>();

        let buffer = ffi::av_memdup(buffer.as_ptr() as *const std::ffi::c_void, size);
        let ctx: *mut ffi::AVIOContext = ffi::avio_alloc_context(
            buffer as *mut u8,
            size as i32,
            0,
            std::ptr::null_mut(),
            None,
            None,
            None,
        );

        let mut format_ctx: *mut AVFormatContext = ffi::avformat_alloc_context();
        (*format_ctx).pb = ctx;

        match ffi::avformat_open_input(
            &mut format_ctx,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            0 => match ffi::avformat_find_stream_info(format_ctx, std::ptr::null_mut()) {
                r if r >= 0 => Ok(format::context::Input::wrap(format_ctx)),
                e => {
                    ffi::avformat_close_input(&mut format_ctx);
                    Err(ffmpeg_next::Error::from(e))
                }
            },
            e => Err(ffmpeg_next::Error::from(e)),
        }
    }
}
