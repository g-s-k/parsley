(define (fizzbuzz lower upper)
  (cond ((< upper lower) (void))
        ((and (eq? (remainder lower 3) 0) (eq? (remainder lower 5) 0))
         (display "fizzbuzz\n") (fizzbuzz (+ lower 1) upper))
        ((eq? (remainder lower 3) 0) (display "fizz\n") (fizzbuzz (+ lower 1) upper))
        ((eq? (remainder lower 5) 0) (display "buzz\n") (fizzbuzz (+ lower 1) upper))
        (else (display lower) (display "\n") (fizzbuzz (+ lower 1) upper))))

(fizzbuzz 1 99)
