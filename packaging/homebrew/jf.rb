class Jf < Formula
  desc "LLM-optimized JSON formatter for better readability and token efficiency"
  homepage "https://github.com/luw2007/llm_json_formatter"
  license "MIT"
  version "0.1.3"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/luw2007/llm_json_formatter/releases/download/v#{version}/jf-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/luw2007/llm_json_formatter/releases/download/v#{version}/jf-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/luw2007/llm_json_formatter/releases/download/v#{version}/jf-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    else
      url "https://github.com/luw2007/llm_json_formatter/releases/download/v#{version}/jf-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    bin.install "jf"
  end

  test do
    assert_match "Format JSON", shell_output("#{bin}/jf --help")
  end
end
