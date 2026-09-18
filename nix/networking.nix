{ node, secrets, lib, ... }: {
  networking = {
    hostName = node.name;
    # The base nixosInstaller function enables network manager which clashes 
    # with the static wifi setup
    networkmanager.enable = lib.mkForce false;
    wireless = {
      enable = true;
      networks.${secrets.wifi.SSID}.psk = secrets.wifi.psk;
    };
  };

  # Publish the hostname via avahi
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    publish = {
      enable = true;
      addresses = true;
      workstation = true;
    };
  };
}
