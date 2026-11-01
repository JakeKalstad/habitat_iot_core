# update and development libs
sudo apt update
sudo apt install -y build-essential curl gcc libssl-dev pkg-config
sudo apt install -y git
# install databases
sudo apt install -y sqlite3 

# rust stuff
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.bashrc
rustc --version


cargo add duckdb --features bundled

# install espup

cargo install espup
espup install
. $HOME/export-esp.sh

