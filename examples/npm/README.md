# parsley_scheme

[![Build Status](https://travis-ci.org/g-s-k/parsley.svg?branch=master)](https://travis-ci.org/g-s-k/parsley)
[![NPM](https://nodei.co/npm/parsley_scheme.png?mini=true)](https://nodei.co/npm/parsley_scheme/)

A WASM implementation of Scheme. Exposes a class `Context`.

## Example

```javascript
const ctx = new Context();

const code = "
(define (sum-to n)
  (if (= n 0) 0 (+ n (sum-to (sub1 n)))))

(sum-to 5)
";

console.log(ctx.run(code));
// 15
```
