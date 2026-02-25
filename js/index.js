import { sum } from './math.js';

print("Hello from JS Module!");
setTimeout(() => {
    print("Let us add!");
    print("10 + 20 = " + sum(10, 20));
}, 3000);
