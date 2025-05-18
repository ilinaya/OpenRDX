import { Component } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { AdminService } from '../../../../shared/services/admin.service';
import { Router } from '@angular/router';
import {TranslatePipe} from '@ngx-translate/core';

@Component({
  selector: 'app-change-password',
  templateUrl: './change-password.component.html',
  imports: [
    TranslatePipe,
    ReactiveFormsModule,
  ],
  styleUrls: ['./change-password.component.scss'],
})
export class ChangePasswordComponent {
  passwordForm: FormGroup;
  submitting = false;
  error: string | null = null;
  success = false;
  showOldPassword = false;
  showNewPassword = false;
  showConfirmPassword = false;

  constructor(
    private fb: FormBuilder,
    private adminService: AdminService,
    public router: Router
  ) {
    this.passwordForm = this.fb.group({
      old_password: ['', [Validators.required]],
      new_password: ['', [Validators.required, Validators.minLength(8)]],
      confirm_password: ['', [Validators.required]]
    }, {
      validators: this.passwordMatchValidator
    });
  }

  passwordMatchValidator(form: FormGroup) {
    const newPassword = form.get('new_password')?.value;
    const confirmPassword = form.get('confirm_password')?.value;

    if (newPassword && confirmPassword && newPassword !== confirmPassword) {
      form.get('confirm_password')?.setErrors({ passwordMismatch: true });
      return { passwordMismatch: true };
    }
    return null;
  }

  togglePasswordVisibility(field: 'old' | 'new' | 'confirm') {
    switch (field) {
      case 'old':
        this.showOldPassword = !this.showOldPassword;
        break;
      case 'new':
        this.showNewPassword = !this.showNewPassword;
        break;
      case 'confirm':
        this.showConfirmPassword = !this.showConfirmPassword;
        break;
    }
  }

  onSubmit() {
    if (this.passwordForm.valid) {
      this.submitting = true;
      this.error = null;
      this.success = false;

      const { old_password, new_password } = this.passwordForm.value;

      this.adminService.changePassword(old_password, new_password).subscribe({
        next: () => {
          this.success = true;
          this.passwordForm.reset();
          setTimeout(() => {
            this.router.navigate(['/settings']);
          }, 2000);
        },
        error: (error) => {
          this.error = error.error?.message || 'Failed to change password';
          this.submitting = false;
        }
      });
    }
  }

  isFieldInvalid(fieldName: string): boolean {
    const field = this.passwordForm.get(fieldName);
    return field ? field.invalid && (field.dirty || field.touched) : false;
  }

  getErrorMessage(fieldName: string): string {
    const field = this.passwordForm.get(fieldName);
    if (!field) return '';

    if (field.errors?.['required']) {
      return `${fieldName.replace('_', ' ')} is required`;
    }
    if (field.errors?.['minlength']) {
      return 'Password must be at least 8 characters long';
    }
    if (field.errors?.['passwordMismatch']) {
      return 'Passwords do not match';
    }
    return '';
  }

  onCancel() {
    this.router.navigate(['/settings']);
  }
}
