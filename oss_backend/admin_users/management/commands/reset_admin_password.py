"""
Django management command to reset admin user password.
Usage: python manage.py reset_admin_password <email> <new_password>
"""
from django.core.management.base import BaseCommand, CommandError
from django.contrib.auth import get_user_model
from django.contrib.auth.password_validation import validate_password
from django.core.exceptions import ValidationError

UserModel = get_user_model()


class Command(BaseCommand):
    help = 'Reset password for an admin user by email'

    def add_arguments(self, parser):
        parser.add_argument('email', type=str, help='Email of the admin user')
        parser.add_argument('password', type=str, help='New password to set')

    def handle(self, *args, **options):
        email = options['email']
        password = options['password']

        try:
            user = UserModel.objects.get(email=email)
        except UserModel.DoesNotExist:
            raise CommandError(f'Admin user with email "{email}" does not exist.')

        # Validate the password
        try:
            validate_password(password, user)
        except ValidationError as e:
            raise CommandError(f'Password validation failed: {"; ".join(e.messages)}')

        # Set the password using Django's password hashing
        user.set_password(password)
        user.save()

        self.stdout.write(
            self.style.SUCCESS(
                f'Successfully reset password for admin user "{email}" (ID: {user.id})'
            )
        )
        self.stdout.write(
            f'Username: {user.username}'
        )
        self.stdout.write(
            f'Email: {user.email}'
        )
        self.stdout.write(
            f'Password hash (first 50 chars): {user.password[:50]}...'
        )

