import datetime
import uuid
from django.conf import settings
from django.core.mail import send_mail
from django.template.loader import render_to_string
from django.utils.html import strip_tags
from django.urls import reverse
from django.utils import timezone
from .models import AdminUser


def generate_token():
    """Generate a unique token for password reset or invitation."""
    return str(uuid.uuid4())


def send_invitation_email(user, request=None):
    """
    Send an invitation email to a new admin user.
    
    Args:
        user: The AdminUser instance to send the invitation to
        request: The HTTP request object (optional, for building absolute URLs)
    
    Returns:
        bool: True if the email was sent successfully, False otherwise
    """
    # Generate a token for the invitation
    token = generate_token()
    
    # Store the token and expiration time in the user's profile or a separate model
    # For simplicity, we're using a simple approach here
    user.invitation_token = token
    user.invitation_expires = timezone.now() + datetime.timedelta(hours=48)
    user.save(update_fields=['invitation_token', 'invitation_expires'])
    
    # Build the set password URL
    if request:
        base_url = request.build_absolute_uri('/').rstrip('/')
    else:
        base_url = settings.BASE_URL if hasattr(settings, 'BASE_URL') else 'http://localhost:8000'
    
    set_password_url = f"{base_url}/set-password/{token}/"
    
    # Prepare the email context
    context = {
        'user': user,
        'set_password_url': set_password_url,
        'current_year': datetime.datetime.now().year
    }
    
    # Render the email templates
    html_message = render_to_string('admin_users/emails/invitation.html', context)
    plain_message = strip_tags(html_message)
    
    # Send the email
    try:
        send_mail(
            subject="You've Been Invited to Join Our Platform",
            message=plain_message,
            from_email=settings.DEFAULT_FROM_EMAIL,
            recipient_list=[user.email],
            html_message=html_message,
            fail_silently=False,
        )
        return True
    except Exception as e:
        print(f"Error sending invitation email: {e}")
        return False


def send_password_reset_email(user, request=None):
    """
    Send a password reset email to an admin user.
    
    Args:
        user: The AdminUser instance to send the password reset to
        request: The HTTP request object (optional, for building absolute URLs)
    
    Returns:
        bool: True if the email was sent successfully, False otherwise
    """
    # Generate a token for the password reset
    token = generate_token()
    
    # Store the token and expiration time in the user's profile or a separate model
    user.reset_token = token
    user.reset_expires = timezone.now() + datetime.timedelta(hours=24)
    user.save(update_fields=['reset_token', 'reset_expires'])
    
    # Build the reset password URL
    if request:
        base_url = request.build_absolute_uri('/').rstrip('/')
    else:
        base_url = settings.BASE_URL if hasattr(settings, 'BASE_URL') else 'http://localhost:8000'
    
    reset_password_url = f"{base_url}/reset-password/{token}/"
    
    # Prepare the email context
    context = {
        'user': user,
        'reset_password_url': reset_password_url,
        'current_year': datetime.datetime.now().year
    }
    
    # Render the email templates
    html_message = render_to_string('admin_users/emails/password_reset.html', context)
    plain_message = strip_tags(html_message)
    
    # Send the email
    try:
        send_mail(
            subject="Password Reset Request",
            message=plain_message,
            from_email=settings.DEFAULT_FROM_EMAIL,
            recipient_list=[user.email],
            html_message=html_message,
            fail_silently=False,
        )
        return True
    except Exception as e:
        print(f"Error sending password reset email: {e}")
        return False