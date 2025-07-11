{
  testPkg = {
    pname = "test-package";
    version = "1.0.0";
    dependencies = [ "dep1" "dep2" ];
  };
  
  dep1 = {
    pname = "dependency-1";
    version = "0.5.0";
    dependencies = [ "base" ];
  };
  
  dep2 = {
    pname = "dependency-2";
    version = "0.3.0";
    dependencies = [ "base" "dep1" ];
  };
  
  base = {
    pname = "base-package";
    version = "2.0.0";
    dependencies = [];
  };
}