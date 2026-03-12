class Ytmusic < Formula
  desc "Keyboard-driven terminal UI client for YouTube Music"
  homepage "https://github.com/SushanthK07/ytmusic"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-aarch64"
      sha256 "ed5288e2ae22cff77da46d61875f41c2507cd64d9cb6d19d3b8ae98104e60244"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-macos-x86_64"
      sha256 "ba95954dfa2bb00f138f50039f92150dd4094fbc59982f54ad0653b5a2046bba"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-aarch64"
      sha256 "bbb54f260a6423f8d9d8d672870a42e3fddc5e8cca169173c58ea5a81ea764b6"
    end

    on_intel do
      url "https://github.com/SushanthK07/ytmusic/releases/download/v#{version}/ytmusic-linux-x86_64"
      sha256 "99e74467838e06e6e8e8febd6c65e68403643bd3c57ecebfeef1aa8f747fd8d5"
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
