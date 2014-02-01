/*
 * Copyright (c) 2014 Kohei Suzuki
 *
 * MIT License
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
 * LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
 * OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
 * WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#include <stdio.h>
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/avutil.h>

static const int TS_PACKET_SIZE = 188;

#define FAIL_IF_ERROR(s) err = (s); if (err < 0) goto fail

static int find_main_streams(const AVFormatContext *ic, AVStream **in_audio, AVStream **in_video)
{
  /* Select audio and video in the most small program id */
  static const int INVALID_PROGRAM_ID = 1000000000;
  int program_id = INVALID_PROGRAM_ID;
  unsigned i;
  for (i = 0; i < ic->nb_programs; i++) {
    AVProgram *program = ic->programs[i];
    AVStream *audio = NULL, *video = NULL;
    unsigned j;
    if (program_id < program->id) {
      continue;
    }
    for (j = 0; j < program->nb_stream_indexes; j++) {
      AVStream *stream = ic->streams[program->stream_index[j]];
      switch (stream->codec->codec_type) {
        case AVMEDIA_TYPE_AUDIO:
          audio = stream;
          break;
        case AVMEDIA_TYPE_VIDEO:
          video = stream;
          break;
        default:
          break;
      }
    }
    if (audio != NULL && video != NULL) {
      *in_audio = audio;
      *in_video = video;
      program_id = program->id;
    }
  }
  if (program_id == INVALID_PROGRAM_ID) {
    return AVERROR_STREAM_NOT_FOUND;
  } else {
    return 0;
  }
}

static int clean_ts(const char *infile, const char *outfile, int64_t npackets)
{
  AVFormatContext *ic = NULL, *oc = NULL;
  int err = 0;
  AVStream *in_audio = NULL, *in_video = NULL;

  FAIL_IF_ERROR(avformat_open_input(&ic, infile, NULL, NULL));
  avio_seek(ic->pb, npackets * TS_PACKET_SIZE, SEEK_SET);
  FAIL_IF_ERROR(avformat_find_stream_info(ic, NULL));

  av_log_set_level(AV_LOG_ERROR);

  FAIL_IF_ERROR(find_main_streams(ic, &in_audio, &in_video));

  FAIL_IF_ERROR(avformat_alloc_output_context2(&oc, NULL, NULL, outfile));
  AVStream *out_video = avformat_new_stream(oc, in_video->codec->codec);
  AVStream *out_audio = avformat_new_stream(oc, in_audio->codec->codec);
  FAIL_IF_ERROR(avcodec_copy_context(out_video->codec, in_video->codec));
  FAIL_IF_ERROR(avcodec_copy_context(out_audio->codec, in_audio->codec));
  if (oc->oformat->flags & AVFMT_GLOBALHEADER) {
    out_video->codec->flags |= CODEC_FLAG_GLOBAL_HEADER;
    out_audio->codec->flags |= CODEC_FLAG_GLOBAL_HEADER;
  }

  if (!(oc->oformat->flags & AVFMT_NOFILE)) {
    FAIL_IF_ERROR(avio_open(&oc->pb, outfile, AVIO_FLAG_WRITE));
  }

  FAIL_IF_ERROR(avformat_write_header(oc, NULL));
  AVPacket packet;
  while ((err = av_read_frame(ic, &packet)) >= 0) {
    const AVStream *in_stream = ic->streams[packet.stream_index];
    AVStream *out_stream = NULL;
    if (in_stream == in_video) {
      out_stream = out_video;
    } else if (in_stream == in_audio) {
      out_stream = out_audio;
    }
    if (out_stream != NULL) {
      packet.stream_index = out_stream->index;
      packet.pts = av_rescale_q_rnd(packet.pts, in_stream->time_base, out_stream->time_base, AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX);
      packet.dts = av_rescale_q_rnd(packet.dts, in_stream->time_base, out_stream->time_base, AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX);
      packet.duration = av_rescale_q_rnd(packet.duration, in_stream->time_base, out_stream->time_base, AV_ROUND_NEAR_INF | AV_ROUND_PASS_MINMAX);
      packet.pos = -1;
      err = av_interleaved_write_frame(oc, &packet);
      if (err < 0) {
        fprintf(stderr, "av_interleaved_write_frame(): Ignore error %s (at %"PRId64")\n", av_err2str(err), avio_tell(ic->pb));
      }
    }
    av_free_packet(&packet);
  }
  if (err != AVERROR_EOF) {
    goto fail;
  }
  FAIL_IF_ERROR(av_write_trailer(oc));

fail:
  avformat_close_input(&ic);
  if (oc != NULL && oc->pb != NULL) {
    avio_close(oc->pb);
  }
  avformat_free_context(oc);
  return err;
}

static const int HD = 0x01, SD = 0x02;
static const int HD_WIDTH = 1440, SD_WIDTH = 720;

static int detect_hd_sd(const char *infile, int64_t npackets)
{
  AVFormatContext *ic = NULL;
  int err = 0;
  int ret = 0;
  unsigned i;

  FAIL_IF_ERROR(avformat_open_input(&ic, infile, NULL, NULL));
  avio_seek(ic->pb, npackets * TS_PACKET_SIZE, SEEK_SET);
  FAIL_IF_ERROR(avformat_find_stream_info(ic, NULL));

  for (i = 0; i < ic->nb_streams; i++) {
    const AVCodecContext *cc = ic->streams[i]->codec;
    if (cc->codec_type == AVMEDIA_TYPE_VIDEO
        && cc->codec_id == AV_CODEC_ID_MPEG2VIDEO) {
      if (cc->width == HD_WIDTH) {
        ret |= HD;
      } else if (cc->width == SD_WIDTH) {
        ret |= SD;
      }
    }
  }

fail:
  avformat_close_input(&ic);
  return ret;
}

static int has_stray_audio(const char *infile, int64_t npackets)
{
  AVFormatContext *ic = NULL;
  int err = 0;
  int ret = 0;
  uint8_t *found_streams = NULL;

  FAIL_IF_ERROR(avformat_open_input(&ic, infile, NULL, NULL));
  avio_seek(ic->pb, npackets * TS_PACKET_SIZE, SEEK_SET);
  FAIL_IF_ERROR(avformat_find_stream_info(ic, NULL));

  /* When an audio stream is found outside the programs, ffmpeg seems to fail
   * with the following error message.
   *
   * [mpegts @ 0x???????] AAC bitstream not in ADTS format and extradata missing
   * av_interleaved_write_frame(): Invalid data found when processing input
   *
   * We have to avoid this kind of error.
   */

  found_streams = av_mallocz(ic->nb_streams);
  unsigned i;
  for (i = 0; i < ic->nb_programs; i++) {
    const AVProgram *program = ic->programs[i];
    unsigned j;
    for (j = 0; j < program->nb_stream_indexes; j++) {
      found_streams[program->stream_index[j]] = 1;
    }
  }

  for (i = 0; i < ic->nb_streams; i++) {
    if (!found_streams[i] && ic->streams[i]->codec->codec_type == AVMEDIA_TYPE_AUDIO) {
      ret = 1;
      break;
    }
  }

fail:
  avformat_close_input(&ic);
  av_free(found_streams);
  return ret;
}

static int higher_p(const char *infile, int64_t npackets, int higher_is_hd)
{
  if (has_stray_audio(infile, npackets)) {
    fprintf(stderr, "%s: Stray audio is found at %"PRId64"*188\n", infile, npackets);
    return 1;
  } else {
    const int r = detect_hd_sd(infile, npackets);
    if ((r & HD) && (r & SD)) {
      // If both are found, proper cutpoint is higher.
      return 1;
    } else if (r & HD) {
      return !higher_is_hd;
    } else if (r & SD) {
      return higher_is_hd;
    } else {
      fprintf(stderr, "%s: Neither HD nor SD at %"PRId64"\n", infile, npackets);
      return 1;
    }
  }
}

static int64_t find_cutpoint(const char *infile, int64_t lo, int64_t hi, int higher_is_hd)
{
  while (lo < hi) {
    const int64_t mid = (lo + hi) / 2;
    const int r = higher_p(infile, mid, higher_is_hd);
    if (r < 0) {
      return r;
    } else if (r) {
      lo = mid+1;
    } else {
      hi = mid;
    }
  }
  return lo;
}

int main(int argc, char *argv[])
{
  if (argc != 3) {
    fprintf(stderr, "Usage: %s input.ts output.ts\n", argv[0]);
    return 1;
  }
  const char *infile = argv[1], *outfile = argv[2];
  av_register_all();
  av_log_set_level(AV_LOG_FATAL);

  static const int MAX_PACKETS = 200000;
  const int begin_hd = detect_hd_sd(infile, 0) & HD, end_hd = detect_hd_sd(infile, MAX_PACKETS) & HD;
  int err;
  if (begin_hd) {
    if (end_hd) {
      err = clean_ts(infile, outfile, 0);
    } else {
      const int64_t npackets = find_cutpoint(infile, 0, MAX_PACKETS, 0);
      if (npackets < 0) {
        err = npackets;
      } else {
        err = clean_ts(infile, outfile, npackets);
      }
    }
  } else {
    if (end_hd) {
      const int64_t npackets = find_cutpoint(infile, 0, MAX_PACKETS, 1);
      if (npackets < 0) {
        err = npackets;
      } else {
        err = clean_ts(infile, outfile, npackets);
      }
    } else {
      err = clean_ts(infile, outfile, 0);
    }
  }

  if (err < 0) {
    fprintf(stderr, "ERROR: %s\n", av_err2str(err));
  }
  return -err;
}
