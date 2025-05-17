import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { VendorService } from '../../../../shared/services/vendor.service';
import { Vendor } from '../../../../shared/models/vendor.model';

@Component({
  selector: 'app-vendor-form',
  templateUrl: './vendor-form.component.html',
  styleUrls: ['./vendor-form.component.scss']
})
export class VendorFormComponent implements OnInit {
  vendorForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private vendorService: VendorService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.vendorForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      vendor_id: ['', [Validators.required, Validators.min(1)]],
      description: ['', [Validators.maxLength(1000)]]
    });
  }

  ngOnInit(): void {
    const vendorId = this.route.snapshot.paramMap.get('id');
    if (vendorId) {
      this.isEditMode = true;
      this.loadVendor(parseInt(vendorId, 10));
    }
  }

  loadVendor(id: number): void {
    this.loading = true;
    this.error = null;

    this.vendorService.getVendor(id).subscribe({
      next: (vendor: Vendor) => {
        this.vendorForm.patchValue(vendor);
        this.loading = false;
      },
      error: (error: Error) => {
        this.error = 'Failed to load vendor details. Please try again.';
        this.loading = false;
      }
    });
  }

  isFieldInvalid(fieldName: string): boolean {
    const field = this.vendorForm.get(fieldName);
    return field ? field.invalid && (field.dirty || field.touched) : false;
  }

  getErrorMessage(fieldName: string): string {
    const field = this.vendorForm.get(fieldName);
    if (!field) return '';

    if (field.errors?.['required']) {
      return `${fieldName.charAt(0).toUpperCase() + fieldName.slice(1)} is required`;
    }
    if (field.errors?.['maxlength']) {
      return `${fieldName.charAt(0).toUpperCase() + fieldName.slice(1)} cannot exceed ${field.errors?.['maxlength'].requiredLength} characters`;
    }
    if (field.errors?.['min']) {
      return `${fieldName.charAt(0).toUpperCase() + fieldName.slice(1)} must be greater than 0`;
    }
    return '';
  }

  onSubmit(): void {
    if (this.vendorForm.invalid) {
      return;
    }

    this.submitting = true;
    this.error = null;

    const vendorData = this.vendorForm.value;

    const request = this.isEditMode
      ? this.vendorService.updateVendor(parseInt(this.route.snapshot.paramMap.get('id')!, 10), vendorData)
      : this.vendorService.createVendor(vendorData);

    request.subscribe({
      next: () => {
        this.router.navigate(['/settings/vendors']);
      },
      error: (error: Error) => {
        this.error = 'Failed to save vendor. Please try again.';
        this.submitting = false;
      }
    });
  }

  onCancel(): void {
    this.router.navigate(['/settings/vendors']);
  }
} 