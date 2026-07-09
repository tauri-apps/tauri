# Copyright 2019-2024 Tauri Programme within The Commons Conservancy
# SPDX-License-Identifier: Apache-2.0
# SPDX-License-Identifier: MIT

# Thin setup.py so CI can build platform-tagged wheels around the prebuilt
# cdylib (copied into tauri_ffi/_native/ before building):
#   TAURI_FFI_VERSION=x.y.z python setup.py bdist_wheel --plat-name macosx_11_0_arm64
# The Python code itself is pure (cffi ABI mode), so wheels are tagged
# py3-none-<platform>: the platform tag only pins the bundled library.

import os

from setuptools import setup

setup(version=os.environ.get("TAURI_FFI_VERSION", "0.0.0"))
