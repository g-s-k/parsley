(define (sqr x) (* x x))
(define (sumsqr x y) (+ (sqr x) (sqr y)))

(define (biggest-2-of-3 x y z)
  (if
   (> y x)
   (if (> z x) (list z y) (list x y))
   (if (> z y) (list z x) (list x y))
   )
  )

(define (big-2-sum-sqrs x y z)
  (define big-2 (biggest-2-of-3 x y z))
  (sumsqr (car big-2) (car (cdr big-2)))
  )
