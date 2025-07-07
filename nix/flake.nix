{
  description = "Information Alchemist Development VM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }: {
    packages.x86_64-linux = {
      hyperv = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "hyperv";

        modules = [
          ./leaf.nix
        ];
      };
    };
  };
}
