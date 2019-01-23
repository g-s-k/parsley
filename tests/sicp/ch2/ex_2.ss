(define make-point cons)
(define x-point car)
(define y-point cdr)

(define (print-point p)
  (display "[")
  (display (x-point p))
  (display ", ")
  (display (y-point p))
  (displayln "]"))

(define make-segment cons)
(define start-segment car)
(define end-segment cdr)

(define (midpoint-segment seg)
  (let ((s (start-segment seg)) (e (end-segment seg)))
      (make-point (/ (+ (x-point s) (x-point e)) 2)
                  (/ (+ (y-point s) (y-point e)) 2))))
