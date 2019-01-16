(define (f-i-calc a b c) (+ a (* 2 b) (* 3 c)))

(define (f-i-helper f-3 f-2 f-1 m)
  (if
   (= m n)
   (f-i-calc f-1 f-2 f-3)
   (f-i-helper f-2 f-1 (f-i-calc f-1 f-2 f-3) (+ m 1))
   )
  )

(define (f-i n)
  (f-i-helper 0 1 2 3)
  )
