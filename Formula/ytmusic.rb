class Ytmusic < Formula
  desc "Keyboard-driven terminal UI client for YouTube Music"
  homepage "https://github.com/SushanthK07/ytmusic"
  version "0.3.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-aarch64"
      sha256 "c8f8cd41fd077981e452992c9f32147a865296e4d0a4a13db5d98add09c4fce9"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-x86_64"
      sha256 "6b115073d9b758c95b31f0fe642d0efdc10f1bab1ee6bcf76d50fdebcbf5640b"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-aarch64"
      sha256 "1972b3106d0332ab74f1bf6a479c1109b27883d84c421898464c9773411a3685"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-x86_64"
      sha256 "fe06b8c3f911ca7f58db89c1ebe5955832c6f4300a11edf09ea03809d5d2ccac"
    end
  end

  depends_on "mpv"
  depends_on "yt-dlp"

  def install
    binary = Dir["ytmusic-*"].first || "ytmusic"
    bin.install binary => "ytmusic"
  end

  test do
    assert_match "ytmusic", shell_output("#{bin}/ytmusic --version 2>&1", 0)
  end
end
