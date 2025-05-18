import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { SecretService } from '../../../../shared/services/secret.service';
import { VendorService } from '../../../../shared/services/vendor.service';
import { Nas, NasGroup, Secret, Vendor } from '../../../../shared/models/nas.model';
import { forkJoin, of } from 'rxjs';
import { catchError, switchMap, tap } from 'rxjs/operators';

@Component({
  selector: 'app-nas-form',
  templateUrl: './nas-form.component.html',
  styleUrls: ['./nas-form.component.scss']
})
export class NasFormComponent implements OnInit {
  nasForm: FormGroup;
  isEditMode = false;
  nasId: number | null = null;
  loading = false;
  submitting = false;
  error = '';
  nasGroups: NasGroup[] = [];
  flattenedGroups: NasGroup[] = [];
  secrets: Secret[] = [];
  vendors: Vendor[] = [];

  constructor(
    private fb: FormBuilder,
    private nasService: NasService,
    private secretService: SecretService,
    private vendorService: VendorService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.nasForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      ip_address: ['', [Validators.required, Validators.pattern('^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$')]],
      coa_enabled: [false],
      coa_port: [3799, [Validators.min(1), Validators.max(65535)]],
      group_ids: [[]],
      secret_id: [null, [Validators.required]],
      vendor_id: [null, [Validators.required]]
    });
  }

  ngOnInit(): void {
    this.loading = true;
    console.log('Loading NAS form data...');

    // Load NAS groups, secrets, and vendors
    forkJoin({
      groups: this.nasService.getNasGroupTree().pipe(
        tap(groups => {
          console.log('Loaded NAS groups:', groups);
          this.nasGroups = groups;
          this.flattenGroups(groups);
        }),
        catchError(error => {
          console.error('Error loading NAS groups:', error);
          return of([]);
        })
      ),
      secrets: this.secretService.listSecrets().pipe(
        tap(secrets => console.log('Loaded secrets:', secrets)),
        catchError(error => {
          console.error('Error loading secrets:', error);
          return of([]);
        })
      ),
      vendors: this.vendorService.getAllVendorsList().pipe(
        tap(vendors => console.log('Loaded vendors:', vendors)),
        catchError(error => {
          console.error('Error loading vendors:', error);
          return of([]);
        })
      )
    }).subscribe({
      next: (result) => {
        console.log('Form data loaded successfully:', result);
        this.secrets = result.secrets;
        this.vendors = result.vendors;
        
        // Check if we're in edit mode
        const id = this.route.snapshot.paramMap.get('id');
        if (id && id !== 'new') {
          this.isEditMode = true;
          this.nasId = +id;
          this.loadNasDetails(+id);
        } else {
          this.loading = false;
        }
      },
      error: (err) => {
        console.error('Error in forkJoin:', err);
        this.error = 'Failed to load form data. Please try again later.';
        this.loading = false;
      }
    });
  }

  private flattenGroups(groups: NasGroup[], level: number = 0): void {
    groups.forEach(group => {
      // Add the current group with its level
      this.flattenedGroups.push({
        ...group,
        level: level // Explicitly set level to ensure it's not undefined
      });
      
      // Recursively add children if they exist
      if (group.children && group.children.length > 0) {
        this.flattenGroups(group.children, level + 1);
      }
    });
  }

  loadNasDetails(id: number): void {
    this.nasService.getNasById(id)
      .subscribe({
        next: (nas) => {
          // Populate the form with NAS details
          this.nasForm.patchValue({
            name: nas.name,
            description: nas.description,
            ip_address: nas.ip_address,
            coa_enabled: nas.coa_enabled,
            coa_port: nas.coa_port,
            group_ids: nas.groups.map(g => g.id),
            secret_id: nas.secret_id || null,
            vendor_id: nas.vendor_id || null
          });
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load NAS details. Please try again later.';
          console.error('Error loading NAS details:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.nasForm.invalid) {
      // Mark all fields as touched to trigger validation messages
      Object.keys(this.nasForm.controls).forEach(key => {
        const control = this.nasForm.get(key);
        control?.markAsTouched();
      });
      return;
    }

    this.submitting = true;
    this.error = '';

    const nasData = this.nasForm.value;
    console.log('Submitting NAS data:', nasData);

    if (this.isEditMode && this.nasId) {
      // Update existing NAS - send all form values
      this.nasService.updateNas(this.nasId, {
        name: nasData.name,
        description: nasData.description,
        ip_address: nasData.ip_address,
        coa_enabled: nasData.coa_enabled,
        coa_port: nasData.coa_port,
        group_ids: nasData.group_ids,
        secret_id: nasData.secret_id,
        vendor_id: nasData.vendor_id
      }).subscribe({
        next: (nas) => {
          console.log('NAS updated successfully:', nas);
          this.submitting = false;
          if (nas && nas.id) {
            this.router.navigate(['/devices/nas', nas.id]);
          } else {
            this.router.navigate(['/devices/nas']);
          }
        },
        error: (err) => {
          console.error('Error updating NAS:', err);
          this.error = err.error?.message || 'Failed to update NAS. Please try again later.';
          this.submitting = false;
        }
      });
    } else {
      // Create new NAS
      this.nasService.createNas(nasData)
        .subscribe({
          next: (nas) => {
            console.log('NAS created successfully:', nas);
            this.submitting = false;
            if (nas && nas.id) {
              this.router.navigate(['/devices/nas', nas.id]);
            } else {
              this.router.navigate(['/devices/nas']);
            }
          },
          error: (err) => {
            console.error('Error creating NAS:', err);
            this.error = err.error?.message || 'Failed to create NAS. Please try again later.';
            this.submitting = false;
          }
        });
    }
  }

  cancel(): void {
    if (this.isEditMode && this.nasId) {
      this.router.navigate(['/devices/nas', this.nasId]);
    } else {
      this.router.navigate(['/devices/nas']);
    }
  }

  onGroupSelectionChange(event: Event, groupId: number): void {
    const checkbox = event.target as HTMLInputElement;
    const currentValue = this.nasForm.get('group_ids')?.value || [];
    
    if (checkbox.checked) {
      // Add the group ID if it's not already in the array
      if (!currentValue.includes(groupId)) {
        this.nasForm.patchValue({
          group_ids: [...currentValue, groupId]
        });
      }
    } else {
      // Remove the group ID from the array
      this.nasForm.patchValue({
        group_ids: currentValue.filter((id: number) => id !== groupId)
      });
    }
  }
}