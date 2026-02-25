import { sum } from './math.js';

print("Hello from JS Module!");

let resp = await fetch('https://ipinfo.io/json');
let content = await resp.json();
print(JSON.stringify(content, null, 2));

setTimeout(() => {
    print("Let us add!");
    print("10 + 20 = " + sum(10, 20));
}, 3000);
