// OCV Consent Check
const OCVConsentManager = {
    clientId: 'friday-assistant',
    userId: null,
    ocvBackendUrl: 'http://localhost:8000',
    ocvConsentUrl: 'http://localhost:3000',
    
    init: function() {
        // Check if we have a user ID in localStorage
        this.userId = localStorage.getItem('ocvUserId');
        
        // Check if we have a consent token
        const hasConsent = localStorage.getItem('ocvConsentToken');
        
        // If we don't have a user ID, generate one
        if (!this.userId) {
            this.userId = this.generateUserId();
            localStorage.setItem('ocvUserId', this.userId);
        }
        
        return hasConsent;
    },
    
    generateUserId: function() {
        // Simple UUID generator
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c === 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    },
    
    requestConsent: function() {
        // Generate a request ID for this consent request
        const requestId = this.generateUserId();
        localStorage.setItem('ocvConsentRequestId', requestId);
        
        // Redirect to the consent page
        window.location.href = `${this.ocvConsentUrl}/consent?` + 
            `requestId=${requestId}&` +
            `clientId=${this.clientId}&` +
            `userId=${this.userId}&` +
            `redirectUrl=${encodeURIComponent(window.location.href)}`;
    },
    
    checkConsentCallback: function() {
        // Check if we're returning from a consent request
        const urlParams = new URLSearchParams(window.location.search);
        const requestId = urlParams.get('requestId');
        const granted = urlParams.get('granted');
        
        if (requestId && localStorage.getItem('ocvConsentRequestId') === requestId) {
            // Clear the request ID
            localStorage.removeItem('ocvConsentRequestId');
            
            // Check if consent was granted
            if (granted === 'true') {
                // Store the consent token (in a real implementation, this would be a JWT)
                const mockToken = `consent-${this.userId}-${new Date().getTime()}`;
                localStorage.setItem('ocvConsentToken', mockToken);
                
                // Clean up the URL
                window.history.replaceState({}, document.title, window.location.pathname);
                return true;
            } else {
                // Consent was denied
                console.log("OCV access was denied by the user");
                return false;
            }
        }
        
        return null; // No callback detected
    }
};
