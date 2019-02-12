(define reverse-subtract
  (lambda (x y)
    (- y x)))

(define foo
  (let ((x 4))
    (lambda (y) (+ x y))))
