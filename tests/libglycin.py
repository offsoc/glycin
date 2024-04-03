#!/usr/bin/python3

import gi
import os
import os.path

gi.require_version("Gly", "1")
from gi.repository import Gly, Gio

dir = os.path.dirname(os.path.abspath(__file__))

test_image = os.path.join(dir, "test-images/images/color/color.jpg")
file = Gio.File.new_for_path(test_image)
loader = Gly.Loader.new(file)
loader.set_sandbox_selector(Gly.SandboxSelector.AUTO)

image = loader.load()
frame = image.next_frame()
texture = frame.get_texture()

mime_type = image.get_mime_type()
width = texture.get_width()

assert mime_type == "image/jpeg", f"Wrong mime type {mime_type}"
assert width == 600, f"Wrong size: {width} px"