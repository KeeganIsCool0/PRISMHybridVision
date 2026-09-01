// JavaScript for PRISM Recording Console Website
// Currently, most functionality is handled by CSS
// This file is available for any future interactive enhancements

document.addEventListener('DOMContentLoaded', function() {
    // Any initialization code can go here
    console.log('PRISM Website loaded');

    // Typing animation for hero text
    const heroTextElement = document.querySelector('.hero-text');
    if (heroTextElement) {
        const fullText = heroTextElement.textContent;
        heroTextElement.textContent = '';

        let i = 0;
        const timer = setInterval(() => {
            if (i < fullText.length) {
                heroTextElement.textContent += fullText.charAt(i);
                i++;
            } else {
                clearInterval(timer);
            }
        }, 100); // typing speed in milliseconds per character
    }

    // Example: Add active class to current page in navigation
    const currentPage = window.location.pathname.split('/').pop();
    const navLinks = document.querySelectorAll('.nav-links a');

    navLinks.forEach(link => {
        if (link.getAttribute('href') === currentPage ||
            (currentPage === '' && link.getAttribute('href') === 'index.html')) {
            link.classList.add('active');
        }
    });
});

// Function to handle smooth scrolling for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        document.querySelector(this.getAttribute('href')).scrollIntoView({
            behavior: 'smooth'
        });
    });
});