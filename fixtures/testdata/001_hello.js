console.log('Hello World');
let resp = await fetch('https://jsonplaceholder.typicode.com/todos/1');
let json = await resp.json();
console.log(resp);
