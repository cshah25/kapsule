{
  description = "Kapsule Container Engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };

    libraries = with pkgs; [
      webkitgtk_4_1
      gtk3
      cairo
      gdk-pixbuf
      glib
      dbus
      librsvg
    ];

    packages = with pkgs; [
      curl
      wget
      pkg-config
      dbus
      openssl_3
      glib
      gtk3
      libsoup_3
      webkitgtk_4_1
    ];
  in
  {
    packages.${system}.default = pkgs.stdenv.mkDerivation {
      pname = "kapsule";
      version = "0.1.0";
      
      src = pkgs.fetchurl {
        url = "https://github.com/cshah25/kapsule/releases/download/v1.1.3/kapsule_0.1.0_amd64.deb";
        sha256 = "sha256-Gp22fzcyN1O58dh7D9l98qf//1bFA7IXnz6K7pinMzk=";
      };

      nativeBuildInputs = [
        pkgs.autoPatchelfHook
        pkgs.dpkg
        pkgs.wrapGAppsHook3
      ];

      buildInputs = packages ++ libraries;

      unpackPhase = ''
        dpkg-deb -x $src .
      '';

      installPhase = ''
        mkdir -p $out
        cp -r usr/* $out/
      '';
    };

    devShells.${system}.default = pkgs.mkShell {
      buildInputs = packages;

      nativeBuildInputs = with pkgs; [
        pkg-config
        rustc
        cargo
        nodejs
      ];

      shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
        export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
        
        echo "========================================="
        echo "Kapsule NixOS Build Environment Ready!"
        echo "To build Kapsule for NixOS, run:"
        echo "  cd kapsule"
        echo "  npm install"
        echo "  npm run tauri build"
        echo "========================================="
      '';
    };
  };
}
