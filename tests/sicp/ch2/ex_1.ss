(define (add-rat x y)
  (make-rat (+ (* (numer x) (denom y))
               (* (numer y) (denom x)))
            (* (denom x) (denom y))))

(define (mul-rat x y)
  (make-rat (* (numer x) (numer y))
            (* (denom x) (denom y))))

(define (gcd a b)
  (if (zero? b)
      a
      (gcd b (remainder a b))))

(define (make-rat n d)
  (let ((g (gcd n d)))
    (let ((a (/ n g)) (b (/ d g)))
      (cond
       ((and (< 0 a) (< 0 b)) (cons (abs a) (abs b)))
       ((< 0 b) (cons (* -1 a) (abs b)))
       (else (cons a b))))))

(define (numer x) (car x))
(define (denom x) (cdr x))

(define (print-rat x)
  (display (numer x))
  (display #\/)
  (displayln (denom x)))

(define one-half (make-rat 1 2))
(print-rat one-half)

(define one-third (make-rat 1 3))
(print-rat one-third)
