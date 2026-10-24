class Lazygitlab < Formula
  desc "Terminal UI for GitLab"
  homepage "https://github.com/arnaudbletterer/lazygitlab"
  version "0.16.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/arnaudbletterer/lazygitlab/releases/download/v#{version}/lazygitlab-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/arnaudbletterer/lazygitlab/releases/download/v#{version}/lazygitlab-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/arnaudbletterer/lazygitlab/releases/download/v#{version}/lazygitlab-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "lazygitlab"
    pkgshare.install "theme.toml.example"
  end

  test do
    assert_predicate bin/"lazygitlab", :executable?
  end
end
