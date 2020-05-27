#!/bin/sh

# NOTE: this script was modified from the CI script in https://gitlab.kitware.com/utils/rust-git-workarea/
# (permalink: https://gitlab.kitware.com/utils/rust-git-workarea/-/blob/e91132ca965318beb31a573c728c081e448fe94f/.gitlab/ci/sccache.sh)
# which is a project under the MIT license. The original license is reproduced below:
#
# Copyright (c) 2016 Kitware, Inc.
#
# Permission is hereby granted, free of charge, to any
# person obtaining a copy of this software and associated
# documentation files (the "Software"), to deal in the
# Software without restriction, including without
# limitation the rights to use, copy, modify, merge,
# publish, distribute, sublicense, and/or sell copies of
# the Software, and to permit persons to whom the Software
# is furnished to do so, subject to the following
# conditions:
#
# The above copyright notice and this permission notice
# shall be included in all copies or substantial portions
# of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
# ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
# TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
# PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
# SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
# CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
# OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
# IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
# DEALINGS IN THE SOFTWARE.

trap 'exit 1' ERR

readonly version="0.2.12"
readonly sha256sum="26fd04c1273952cc2a0f359a71c8a1857137f0ee3634058b3f4a63b69fc8eb7f"
readonly filename="sccache-$version-x86_64-unknown-linux-musl"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > sccache.sha256sum
curl -OL "https://github.com/mozilla/sccache/releases/download/$version/$tarball"
sha256sum --check sccache.sha256sum
tar xf "$tarball"
mv "$filename/sccache" .
