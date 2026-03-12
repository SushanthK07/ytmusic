class Ytmusic < Formula
  desc "Keyboard-driven terminal UI client for YouTube Music"
  homepage "https://github.com/SushanthK07/ytmusic"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-aarch64"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-x86_64"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-aarch64"
      sha256 "PLACEHOLDER"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-x86_64"
      sha256 "PLACEHOLDER"
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
