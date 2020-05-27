#!/bin/sh

# NOTE: this script was modified from the CI script in https://gitlab.kitware.com/utils/rust-git-workarea/
# (permalink: https://gitlab.kitware.com/utils/rust-git-workarea/-/blob/e91132ca965318beb31a573c728c081e448fe94f/.gitlab/ci/tarpaulin.sh)
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

readonly version="0.11.1"
readonly sha256sum="b51d5c233e1145036c50168d50a4fb3a9f09c19511a04da866f662ed7217eb5f"
readonly filename="cargo-tarpaulin-$version-travis"
readonly tarball="$filename.tar.gz"

cd .gitlab

echo "$sha256sum  $tarball" > tarpaulin.sha256sum
curl -OL "https://github.com/xd009642/tarpaulin/releases/download/$version/$tarball"
sha256sum --check tarpaulin.sha256sum
tar xf "$tarball"
