# clean-ts
FFmpeg や MPlayer でうまく処理できるように MPEG-2 TS ファイルを変換する。

基本的には `ffmpeg -i infile.ts -acodec copy -vcodec copy -y outfile.ts` と同じように、主要なビデオストリームとオーディオストリームのみを残した MPEG-2 TS ファイルを生成する。
それに加えて、TOKYO MX におけるマルチ編成時の SD/HD 切り替えがファイルの先頭付近 (先頭から 200000 パケット) にあった場合、切り替え後から始まるように開始位置を調整する。
