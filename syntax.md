## Example
```
namespace Test;
use std::io;
use pocketmine::world::World;

var test = new Test();

var test2 = Test2::write($test);
```

transpiles to
```
use namespace\Test;
use namespace\Test2;

