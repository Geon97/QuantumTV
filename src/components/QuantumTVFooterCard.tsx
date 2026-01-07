'use client';

import { useEffect, useState } from 'react';

import { useSite } from './SiteProvider';

const QuantumTVFooterCard = () => {
  const [isVisible, setIsVisible] = useState(false);
  const { siteName } = useSite();

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { threshold: 0.1 }
    );

    const element = document.getElementById('deco-footer-card');
    if (element) {
      observer.observe(element);
    }

    return () => {
      if (element) {
        observer.unobserve(element);
      }
    };
  }, []);

  return (
    <div
      id='deco-footer-card'
      className={`relative overflow-hidden transition-all duration-1000 transform ${
        isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-10'
      }`}
    >
    </div>
  );
};

export default QuantumTVFooterCard;
