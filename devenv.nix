{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  packages = with pkgs; [ pkg-config ];

  languages.rust.enable = true;

}
