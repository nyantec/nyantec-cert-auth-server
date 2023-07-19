{ lib
, openssl
, pkg-config
, rustPlatform
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  src = ./.;

  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config ];

  cargoLock.lockFile = ./Cargo.lock;

  meta = with lib; {
    inherit (cargoToml.package) description;
    homepage = cargoToml.package.repository;
    license = licenses.miros;
    maintainers = with maintainers; [ yayayayaka ];
  };
}
