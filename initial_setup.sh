if ! [ -x "$(command -v rustup)" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup
  chmod 755 rustup
  ./rustup -y
  rm rustup
fi

rustup toolchain add nightly
rustup component add rust-src
