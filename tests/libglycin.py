#!/usr/bin/python3

import gi
import os
import os.path
import sys

gi.require_version("Gly", "1")
from gi.repository import Gly, Gio, GLib

def main():
    GLib.timeout_add_seconds(interval = 2, function = cb_exit)

    dir = os.path.dirname(os.path.abspath(__file__))

    test_image = os.path.join(dir, "test-images/images/color/color.jpg")
    file = Gio.File.new_for_path(test_image)

    # Sync tests

    loader = Gly.Loader(file=file)
    loader.set_sandbox_selector(Gly.SandboxSelector.AUTO)

    image = loader.load()
    frame = image.next_frame()
    texture = frame.get_texture()

    mime_type = image.get_mime_type()
    width = texture.get_width()

    assert mime_type == "image/jpeg", f"Wrong mime type {mime_type}"
    assert width == 600, f"Wrong size: {width} px"

    # Async tests

    loader = Gly.Loader(file=file)
    loader.set_sandbox_selector(Gly.SandboxSelector.AUTO)

    image = loader.load_async(None, loader_cb, "loader_data")

    GLib.MainLoop().run()

def loader_cb(loader, result, user_data):
    assert user_data == "loader_data"
    image = loader.load_finish(result)
    image.next_frame_async(None, image_cb, "image_data")

def image_cb(image, result, user_data):
    assert user_data == "image_data"
    frame = image.next_frame_finish(result)

    assert image.get_mime_type() == "image/jpeg"
    sys.exit(0)

def cb_exit():
    print("Test: Exiting after predefined waiting time.", file=sys.stderr)
    sys.exit(1)

if __name__ == "__main__":
    main()