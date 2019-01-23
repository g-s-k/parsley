(define make-point cons)
(define x-point car)
(define y-point cdr)

(define (make-rect p w h)
  (cons p (cons w h)))
(define rect-origin car)
(define rect-dims cdr)
(define (rect-width r) (car (rect-dims r)))
(define (rect-height r) (cdr (rect-dims r)))

(define (rect-area r) (* (rect-width r) (rect-height r)))
(define (rect-perimeter r) (* 2 (+ (rect-width r) (rect-height r))))
