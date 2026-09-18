{ secrets, lib, ... }: {
  security.sudo.wheelNeedsPassword = false;
  users = {
    mutableUsers = false;
    users = {
      # Default user
      alex = {
        isNormalUser = true;
        extraGroups = [ "wheel" ];
        hashedPassword = secrets.password;
        openssh.authorizedKeys.keys = secrets.authorizedKeys;
      };
      # By default the root account is accessible. Set an invalid password hash
      # to disable it
      root = { hashedPassword = lib.mkForce "!"; };
      # Config merging is purely additive so we can't actually remove the nixos
      # user but we can make it unusable (in the base config it wouldn't respond
      # to ssh anyway)
      nixos = lib.mkForce {
        isSystemUser = lib.mkForce true;
        isNormalUser = lib.mkForce false;
        createHome = lib.mkForce false;
        hashedPassword = lib.mkForce "!";
        shell = lib.mkForce "/usr/sbin/nologin";
        group = "nogroup";
        openssh.authorizedKeys.keys = lib.mkForce [ ];
      };
    };
  };
  # Also prevent getty from trying to autologin (nologin will make it fail
  # and loop forever
  services.getty.autologinUser = lib.mkForce null;
  systemd.services."serial-getty@ttyAMA0".enable = lib.mkForce false;
}
