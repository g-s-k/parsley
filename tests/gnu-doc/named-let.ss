(define (>= a b) (or (> a b) (= a b)))

(let loop
    ((numbers '(3 -2 1 6 -5))
     (nonneg '())
     (neg '()))
  (cond ((null? numbers)
         (list nonneg neg))
        ((>= (car numbers) 0)
         (loop (cdr numbers)
               (cons (car numbers) nonneg)
               neg))
        (else
         (loop (cdr numbers)
               nonneg
               (cons (car numbers) neg)))))
