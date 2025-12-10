;; Civic-Connect - Guix Package Definition
;; Run: guix shell -D -f guix.scm

(use-modules (guix packages)
             (guix gexp)
             (guix git-download)
             (guix build-system gnu)
             ((guix licenses) #:prefix license:)
             (gnu packages base))

(define-public civic_connect
  (package
    (name "Civic-Connect")
    (version "0.1.0")
    (source (local-file "." "Civic-Connect-checkout"
                        #:recursive? #t
                        #:select? (git-predicate ".")))
    (build-system gnu-build-system)
    (synopsis "Guix channel/infrastructure")
    (description "Guix channel/infrastructure - part of the RSR ecosystem.")
    (home-page "https://github.com/hyperpolymath/Civic-Connect")
    (license license:agpl3+)))

;; Return package for guix shell
civic_connect
