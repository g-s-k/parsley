(define (f-r n)
  (if
   (< n 3)
   n
   (+ (f-r (- n 1)) (* 2 (f-r (- n 2))) (* 3 (f-r (- n 3))))
   )
  )
