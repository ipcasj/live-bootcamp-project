import { test, expect } from '@playwright/test';

test('Debug DOM structure and click events', async ({ page }) => {
  // Navigate to the HTML file directly
  const htmlPath = 'file:///Users/igor/rust_projects/lgr/live-bootcamp-project/auth-service/assets/index.html';
  await page.goto(htmlPath);
  
  // First, let's examine the actual DOM structure
  const domStructure = await page.evaluate(() => {
    console.log('=== EXAMINING DOM STRUCTURE ===');
    
    // Check navigation links
    const navLinks = document.querySelectorAll('nav a');
    console.log('Navigation links found:', navLinks.length);
    navLinks.forEach((link, i) => {
      console.log(`Link ${i}: id="${link.id}", href="${link.href}", text="${link.textContent}"`);
    });
    
    // Check sections
    const sections = document.querySelectorAll('section');
    console.log('Sections found:', sections.length);
    sections.forEach((section, i) => {
      console.log(`Section ${i}: id="${section.id}", hidden="${section.classList.contains('hidden')}"`);
    });
    
    // Check AuthApp
    console.log('AuthApp on window:', !!window.authApp);
    console.log('AuthApp class available:', typeof window.AuthApp);
    
    return {
      navLinksCount: navLinks.length,
      sectionsCount: sections.length,
      authAppExists: !!window.authApp,
      authAppClass: typeof window.AuthApp
    };
  });
  
  console.log('DOM structure:', domStructure);
  
  // Try clicking the signup link with different strategies
  console.log('\n=== Testing signup link click ===');
  
  // Strategy 1: Direct click with ID selector
  try {
    await page.click('#signup-link');
    console.log('✅ Direct #signup-link click succeeded');
    
    // Wait a moment and check visibility
    await page.waitForTimeout(100);
    const isVisible = await page.evaluate(() => {
      const section = document.querySelector('#signup-section');
      return section && !section.classList.contains('hidden');
    });
    console.log('Signup section visible after direct click:', isVisible);
    
  } catch (error) {
    console.log('❌ Direct #signup-link click failed:', error.message);
  }
  
  // Strategy 2: Find link by text and click
  try {
    await page.click('text=Sign Up');
    console.log('✅ Text-based "Sign Up" click succeeded');
    
    await page.waitForTimeout(100);
    const isVisible = await page.evaluate(() => {
      const section = document.querySelector('#signup-section');
      return section && !section.classList.contains('hidden');
    });
    console.log('Signup section visible after text click:', isVisible);
    
  } catch (error) {
    console.log('❌ Text-based "Sign Up" click failed:', error.message);
  }
  
  // Strategy 3: Programmatic showView
  const programmaticResult = await page.evaluate(() => {
    console.log('\n=== Testing programmatic showView ===');
    if (window.authApp) {
      try {
        window.authApp.showView('signup-section');
        const section = document.querySelector('#signup-section');
        const isVisible = section && !section.classList.contains('hidden');
        console.log('✅ Programmatic showView succeeded, visible:', isVisible);
        return { success: true, visible: isVisible };
      } catch (error) {
        console.log('❌ Programmatic showView failed:', error.message);
        return { success: false, error: error.message };
      }
    } else {
      console.log('❌ No AuthApp available for programmatic test');
      return { success: false, error: 'No AuthApp' };
    }
  });
  
  console.log('Programmatic result:', programmaticResult);
  
  // Check event bindings
  const eventBindingInfo = await page.evaluate(() => {
    console.log('\n=== Checking Event Bindings ===');
    const signupLink = document.querySelector('#signup-link');
    
    if (signupLink) {
      console.log('Signup link found');
      console.log('Has onclick handler:', !!signupLink.onclick);
      console.log('Has addEventListener handlers:', !!signupLink.listenerCount);
      
      // Check for click event listeners
      const hasListeners = signupLink.cloneNode(true).click !== signupLink.click;
      console.log('Has event listeners (cloneNode test):', hasListeners);
      
      return { 
        linkExists: true, 
        hasOnclick: !!signupLink.onclick,
        hasEventListeners: hasListeners
      };
    } else {
      console.log('❌ Signup link not found');
      return { linkExists: false };
    }
  });
  
  console.log('Event binding info:', eventBindingInfo);
  
  // Final DOM state check
  const finalState = await page.evaluate(() => {
    const loginSection = document.querySelector('#login-section');
    const signupSection = document.querySelector('#signup-section');
    
    return {
      loginVisible: loginSection && !loginSection.classList.contains('hidden'),
      signupVisible: signupSection && !signupSection.classList.contains('hidden'),
      loginHidden: loginSection && loginSection.classList.contains('hidden'),
      signupHidden: signupSection && signupSection.classList.contains('hidden')
    };
  });
  
  console.log('\n=== Final DOM state ===');
  console.log('Final DOM state:', finalState);
});