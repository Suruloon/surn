## Example
```
namespace Test;
import { Test, Test2 } from "namespace";

var test = new Test();

var test2 = Test2::write($test);
```

transpiles to
```
use namespace\Test;
use namespace\Test2;

