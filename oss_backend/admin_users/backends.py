import logging
from django.contrib.auth.backends import ModelBackend
from django.contrib.auth import get_user_model
from django.db.models import Q

logger = logging.getLogger(__name__)
UserModel = get_user_model()


class EmailBackend(ModelBackend):
    """
    Authentication backend that uses email instead of username.
    """
    def authenticate(self, request, username=None, password=None, **kwargs):
        logger.debug("=" * 80)
        logger.debug("EmailBackend.authenticate() called")
        logger.debug(f"username parameter: {username}")
        logger.debug(f"kwargs: {kwargs}")
        
        if username is None:
            username = kwargs.get('email')
            logger.debug(f"Extracted username from kwargs['email']: {username}")
        
        if username is None or password is None:
            logger.warning(f"Missing username or password. username={username}, password={'provided' if password else 'None'}")
            return None
        
        logger.debug(f"Looking up user with email or username: {username}")
        try:
            # Try to get user by email or username
            user = UserModel.objects.get(Q(email=username) | Q(username=username))
            logger.debug(f"User found: id={user.id}, email={user.email}, username={user.username}")
            logger.debug(f"User password hash (first 50 chars): {user.password[:50] if user.password else 'None'}...")
            logger.debug(f"User is_active: {user.is_active}, is_staff: {user.is_staff}")
        except UserModel.DoesNotExist:
            logger.warning(f"No user found with email or username: {username}")
            # Run the default password hasher once to reduce the timing
            # difference between an existing and a non-existing user (#20760).
            UserModel().set_password(password)
            logger.debug("=" * 80)
            return None
        except Exception as e:
            logger.error(f"Exception during user lookup: {str(e)}", exc_info=True)
            logger.debug("=" * 80)
            return None
        
        logger.debug("Checking password...")
        password_valid = user.check_password(password)
        logger.debug(f"Password check result: {password_valid}")
        
        if password_valid:
            logger.debug("Password is valid, checking if user can authenticate...")
            can_authenticate = self.user_can_authenticate(user)
            logger.debug(f"User can authenticate: {can_authenticate}")
            
            if can_authenticate:
                logger.info(f"Authentication SUCCESSFUL for user: {user.id} ({user.email})")
                logger.debug("=" * 80)
                return user
            else:
                logger.warning(f"User {user.id} cannot authenticate (likely inactive)")
        else:
            logger.warning(f"Password check FAILED for user: {user.id} ({user.email})")
            # Try to see what's wrong
            logger.debug(f"Password length provided: {len(password)}")
            logger.debug(f"Stored password hash algorithm: {user.password.split('$')[0] if user.password and '$' in user.password else 'unknown'}")
        
        logger.debug("=" * 80)
        return None

