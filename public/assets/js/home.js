let currentUserType = null;

// Use server-side password verification
async function verifyPassword(userType, password) {
    try {
        const formData = new FormData();
        formData.append('user_type', userType);
        formData.append('password', password);
        
        const response = await fetch('/api/verify-password', {
            method: 'POST',
            body: formData
        });
        
        const result = await response.json();
        return result;
    } catch (error) {
        return { success: false, message: 'Connection error' };
    }
}

function showPasswordModal(userType) {
    currentUserType = userType;
    const modal = document.getElementById('passwordModal');
    const modalTitle = document.getElementById('modalTitle');
    const passwordInput = document.getElementById('passwordInput');
    const errorMessage = document.getElementById('errorMessage');
    
    // Update modal title
    modalTitle.textContent = userType === 'admin' ? 'Administrator Access' : 'Expert Access';
    
    // Reset form
    passwordInput.value = '';
    errorMessage.style.display = 'none';
    
    // Show modal
    modal.style.display = 'block';
    
    // Focus on password input
    setTimeout(() => {
        passwordInput.focus();
    }, 100);
}

function closeModal() {
    const modal = document.getElementById('passwordModal');
    modal.style.display = 'none';
    currentUserType = null;
}

function accessChildren() {
    // Direct access for children - no password required
    window.location.href = '/children';
}

// Handle password form submission
document.getElementById('passwordForm').addEventListener('submit', async function(e) {
    e.preventDefault();
    
    const passwordInput = document.getElementById('passwordInput');
    const errorMessage = document.getElementById('errorMessage');
    const enteredPassword = passwordInput.value;
    
    // Show loading state
    const submitBtn = this.querySelector('button[type="submit"]');
    const originalText = submitBtn.textContent;
    submitBtn.textContent = 'Verifying...';
    submitBtn.disabled = true;
    
    try {
        const result = await verifyPassword(currentUserType, enteredPassword);
        
        if (result.success) {
            window.location.href = result.redirect;
        } else {
            // Incorrect password
            errorMessage.textContent = result.message || 'Incorrect password. Please try again.';
            errorMessage.style.display = 'block';
            passwordInput.value = '';
            passwordInput.focus();
            
            // Add shake animation
            passwordInput.style.animation = 'shake 0.5s ease-in-out';
            setTimeout(() => {
                passwordInput.style.animation = '';
            }, 500);
        }
    } catch (error) {
        errorMessage.textContent = 'Connection error. Please try again.';
        errorMessage.style.display = 'block';
    } finally {
        // Reset button state
        submitBtn.textContent = originalText;
        submitBtn.disabled = false;
    }
});

// Close modal when clicking outside
window.addEventListener('click', function(e) {
    const modal = document.getElementById('passwordModal');
    if (e.target === modal) {
        closeModal();
    }
});

// Close modal with Escape key
document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        closeModal();
    }
});

// Add shake animation CSS
const style = document.createElement('style');
style.textContent = `
    @keyframes shake {
        0%, 100% { transform: translateX(0); }
        10%, 30%, 50%, 70%, 90% { transform: translateX(-5px); }
        20%, 40%, 60%, 80% { transform: translateX(5px); }
    }
`;
document.head.appendChild(style);