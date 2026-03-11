use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use std::env;

pub struct EmailService {
    smtp_host: String,
    smtp_username: String,
    smtp_password: String,
    from_email: String,
    from_name: String,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_string()),
            smtp_username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
            from_email: env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| "noreply@aeroledger.com".to_string()),
            from_name: env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "AeroLedger".to_string()),
        }
    }

    pub async fn send_booking_confirmation(
        &self,
        to_email: &str,
        user_name: &str,
        booking_id: &str,
        flight_details: &str,
        total_price: f64,
        crypto_method: &str,
    ) -> Result<(), String> {
        let subject = format!("Booking Confirmation - {}", booking_id);
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center;">
                    <h1 style="color: white; margin: 0;">AeroLedger</h1>
                    <p style="color: white; margin: 10px 0 0 0;">Luxury Aviation Platform</p>
                </div>
                
                <div style="padding: 30px; background: #f9f9f9;">
                    <h2 style="color: #333;">Booking Confirmed!</h2>
                    <p>Dear {},</p>
                    <p>Your booking has been confirmed. Here are the details:</p>
                    
                    <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
                        <p><strong>Booking ID:</strong> {}</p>
                        <p><strong>Flight:</strong> {}</p>
                        <p><strong>Total Price:</strong> ${:.2}</p>
                        <p><strong>Payment Method:</strong> {}</p>
                    </div>
                    
                    <p>Please complete your payment to finalize the booking.</p>
                    
                    <div style="margin: 30px 0; text-align: center;">
                        <a href="http://localhost:3001/dashboard" 
                           style="background: #667eea; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block;">
                            View Dashboard
                        </a>
                    </div>
                    
                    <p style="color: #666; font-size: 12px; margin-top: 30px;">
                        If you have any questions, please contact our support team.
                    </p>
                </div>
                
                <div style="background: #333; color: white; padding: 20px; text-align: center; font-size: 12px;">
                    <p>&copy; 2025 AeroLedger. All rights reserved.</p>
                </div>
            </body>
            </html>
            "#,
            user_name, booking_id, flight_details, total_price, crypto_method
        );

        self.send_email(to_email, &subject, &body).await
    }

    pub async fn send_order_confirmation(
        &self,
        to_email: &str,
        user_name: &str,
        order_id: &str,
        items: &str,
        total_price: f64,
        wallet_address: &str,
        crypto_amount: f64,
        crypto_method: &str,
    ) -> Result<(), String> {
        let subject = format!("Order Confirmation - {}", order_id);
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center;">
                    <h1 style="color: white; margin: 0;">AeroLedger</h1>
                    <p style="color: white; margin: 10px 0 0 0;">Luxury Marketplace</p>
                </div>
                
                <div style="padding: 30px; background: #f9f9f9;">
                    <h2 style="color: #333;">Order Confirmed!</h2>
                    <p>Dear {},</p>
                    <p>Your order has been placed successfully. Here are the details:</p>
                    
                    <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
                        <p><strong>Order ID:</strong> {}</p>
                        <p><strong>Items:</strong> {}</p>
                        <p><strong>Total Price:</strong> ${:.2}</p>
                    </div>
                    
                    <div style="background: #fff3cd; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #ffc107;">
                        <h3 style="margin-top: 0; color: #856404;">Payment Instructions</h3>
                        <p><strong>Send exactly:</strong> {:.8} {}</p>
                        <p><strong>To address:</strong></p>
                        <p style="background: #f8f9fa; padding: 10px; border-radius: 4px; word-break: break-all; font-family: monospace; font-size: 12px;">
                            {}
                        </p>
                        <p style="color: #856404; font-size: 12px; margin-top: 15px;">
                            ⚠️ Please send the exact amount to avoid payment issues.
                        </p>
                    </div>
                    
                    <div style="margin: 30px 0; text-align: center;">
                        <a href="http://localhost:3001/orders" 
                           style="background: #667eea; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block;">
                            Track Order
                        </a>
                    </div>
                    
                    <p style="color: #666; font-size: 12px; margin-top: 30px;">
                        Your order will be processed after payment confirmation.
                    </p>
                </div>
                
                <div style="background: #333; color: white; padding: 20px; text-align: center; font-size: 12px;">
                    <p>&copy; 2025 AeroLedger. All rights reserved.</p>
                </div>
            </body>
            </html>
            "#,
            user_name, order_id, items, total_price, crypto_amount, crypto_method, wallet_address
        );

        self.send_email(to_email, &subject, &body).await
    }

    pub async fn send_payment_confirmed(
        &self,
        to_email: &str,
        user_name: &str,
        order_id: &str,
        tx_hash: &str,
    ) -> Result<(), String> {
        let subject = "Payment Confirmed - Order Processing";
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center;">
                    <h1 style="color: white; margin: 0;">AeroLedger</h1>
                </div>
                
                <div style="padding: 30px; background: #f9f9f9;">
                    <h2 style="color: #28a745;">✓ Payment Confirmed!</h2>
                    <p>Dear {},</p>
                    <p>We have received your payment and your order is now being processed.</p>
                    
                    <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
                        <p><strong>Order ID:</strong> {}</p>
                        <p><strong>Transaction Hash:</strong></p>
                        <p style="background: #f8f9fa; padding: 10px; border-radius: 4px; word-break: break-all; font-family: monospace; font-size: 11px;">
                            {}
                        </p>
                    </div>
                    
                    <p>You will receive another email once your order is ready for delivery.</p>
                    
                    <div style="margin: 30px 0; text-align: center;">
                        <a href="http://localhost:3001/orders" 
                           style="background: #28a745; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block;">
                            View Order Status
                        </a>
                    </div>
                </div>
                
                <div style="background: #333; color: white; padding: 20px; text-align: center; font-size: 12px;">
                    <p>&copy; 2025 AeroLedger. All rights reserved.</p>
                </div>
            </body>
            </html>
            "#,
            user_name, order_id, tx_hash
        );

        self.send_email(to_email, &subject, &body).await
    }

    pub async fn send_welcome_email(
        &self,
        to_email: &str,
        user_name: &str,
    ) -> Result<(), String> {
        let subject = "Welcome to AeroLedger!";
        let body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; text-align: center;">
                    <h1 style="color: white; margin: 0;">Welcome to AeroLedger!</h1>
                </div>
                
                <div style="padding: 30px; background: #f9f9f9;">
                    <h2 style="color: #333;">Hello, {}!</h2>
                    <p>Thank you for joining AeroLedger, the premier luxury aviation and marketplace platform.</p>
                    
                    <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
                        <h3>What you can do:</h3>
                        <ul style="line-height: 1.8;">
                            <li>✈️ Book private flights with cryptocurrency</li>
                            <li>🛍️ Purchase luxury items (jets, yachts, real estate)</li>
                            <li>💰 Invest in tokenized assets</li>
                            <li>📊 Track your portfolio</li>
                        </ul>
                    </div>
                    
                    <div style="margin: 30px 0; text-align: center;">
                        <a href="http://localhost:3001/marketplace" 
                           style="background: #667eea; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block; margin: 5px;">
                            Explore Marketplace
                        </a>
                        <a href="http://localhost:3001/flights" 
                           style="background: #764ba2; color: white; padding: 12px 30px; 
                                  text-decoration: none; border-radius: 5px; display: inline-block; margin: 5px;">
                            Book a Flight
                        </a>
                    </div>
                    
                    <p style="color: #666; font-size: 12px; margin-top: 30px;">
                        If you have any questions, our support team is here to help.
                    </p>
                </div>
                
                <div style="background: #333; color: white; padding: 20px; text-align: center; font-size: 12px;">
                    <p>&copy; 2025 AeroLedger. All rights reserved.</p>
                </div>
            </body>
            </html>
            "#,
            user_name
        );

        self.send_email(to_email, &subject, &body).await
    }

    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let creds = Credentials::new(
            self.smtp_username.clone(),
            self.smtp_password.clone(),
        );

        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| format!("Failed to send email: {}", e))?;

        Ok(())
    }
}
