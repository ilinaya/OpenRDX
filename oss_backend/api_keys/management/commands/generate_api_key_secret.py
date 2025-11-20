import secrets
import string
import os
from django.core.management.base import BaseCommand
from django.conf import settings


class Command(BaseCommand):
    help = 'Generates a secure random secret for API key JWT signing and saves it to environment'

    def handle(self, *args, **options):
        # Generate a secure random secret (64 characters)
        alphabet = string.ascii_letters + string.digits + string.punctuation
        secret = ''.join(secrets.choice(alphabet) for _ in range(64))

        self.stdout.write(self.style.SUCCESS('Generated API_KEY_JWT_SECRET:'))
        self.stdout.write(self.style.WARNING(secret))
        self.stdout.write(self.style.SUCCESS('\nAdd this to your .env file or environment:'))
        self.stdout.write(f'API_KEY_JWT_SECRET={secret}'))

        # Optionally, try to update .env file if it exists
        env_file = os.path.join(settings.BASE_DIR, '.env')
        if os.path.exists(env_file):
            with open(env_file, 'r') as f:
                content = f.read()
            
            if 'API_KEY_JWT_SECRET' in content:
                # Update existing value
                lines = content.split('\n')
                updated_lines = []
                for line in lines:
                    if line.startswith('API_KEY_JWT_SECRET='):
                        updated_lines.append(f'API_KEY_JWT_SECRET={secret}')
                    else:
                        updated_lines.append(line)
                with open(env_file, 'w') as f:
                    f.write('\n'.join(updated_lines))
                self.stdout.write(self.style.SUCCESS(f'\nUpdated {env_file}'))
            else:
                # Append new value
                with open(env_file, 'a') as f:
                    f.write(f'\nAPI_KEY_JWT_SECRET={secret}\n')
                self.stdout.write(self.style.SUCCESS(f'\nAppended to {env_file}'))

